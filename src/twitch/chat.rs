use anyhow::Result;
use tmi::{Action, Channel, Client, Credentials, Message};
use tokio::{sync::Mutex, time::Instant};
use tracing::{debug, error, info};

use crate::{
	database::log::{Log, LogType},
	global::GlobalState,
};

static THRESHOLD: usize = 200;
static TIME_THRESHOLD: u64 = 20;

pub async fn start(global: GlobalState) -> Result<()> {
	info!("Starting listening to chat messages");

	let logs: Mutex<Vec<Log>> = Mutex::new(Vec::new());
	let mut last_flush = Instant::now();

	let channels = global
		.channels
		.clone()
		.into_iter()
		.map(|c| Channel::parse(format!("#{}", c)))
		.collect::<Result<Vec<_>, _>>()?;

	let credentials = Credentials::anon();

	let mut client = Client::builder().credentials(credentials).connect().await?;
	client.join_all(&channels).await?;

	loop {
		let msg = client.recv().await?;
		let mut logs_vec = logs.lock().await;

		match msg.as_typed()? {
			Message::Privmsg(msg)
				if (!msg.text().starts_with("$") || !msg.text().starts_with("!"))
					&& global
						.ignored_users
						.contains(&msg.sender().name().to_string()) =>
			{
				logs_vec.push(Log {
					channel: msg.channel().get(1..).unwrap().to_string().to_lowercase(),
					content: Some(msg.text().to_string()),
					user_id: Some(msg.sender().id().to_string()),
					username: msg.sender().name().to_string().to_lowercase(),
					..Default::default()
				})
			}
			Message::ClearChat(msg) if msg.action().is_ban() || msg.action().is_time_out() => {
				match msg.action() {
					Action::Ban(ban) => logs_vec.push(Log {
						channel: msg.channel().get(1..).unwrap().to_string().to_lowercase(),
						content: None,
						user_id: Some(ban.id().to_string()),
						username: ban.user().to_string().to_lowercase(),
						log_type: LogType::Ban,
						..Default::default()
					}),
					Action::TimeOut(timeout) => logs_vec.push(Log {
						channel: msg.channel().get(1..).unwrap().to_string().to_lowercase(),
						content: None,
						user_id: Some(timeout.id().to_string()),
						username: timeout.user().to_string().to_lowercase(),
						log_type: LogType::Ban,
						..Default::default()
					}),
					_ => {}
				}
			}
			Message::Reconnect => {
				client.reconnect().await?;
				client.join_all(&channels).await?;
			}
			Message::Ping(ping) => client.pong(&ping).await?,
			_ => {}
		}

		if logs_vec.len() >= THRESHOLD || last_flush.elapsed().as_secs() >= TIME_THRESHOLD {
			let len = logs_vec.len();

			if len == 0 {
				continue;
			}

			match Log::bulk_insert(&global.db, logs_vec.clone()).await {
				Ok(_) => {}
				Err(e) => {
					error!("Error while inserting logs into database: {:?}", e);
				}
			}

			logs_vec.clear();
			last_flush = Instant::now();
			debug!("Flushing logs to database {:?}", len);
		}
	}
}
