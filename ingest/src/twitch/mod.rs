use std::{
	sync::Arc,
	time::{Duration, Instant},
};

use anyhow::Result;
use tmi::{Action, Client, Credentials, Message};
use tokio::sync::Mutex;
use tracing::info;
use utils::database::{Log, LogType};

use crate::{global::GlobalState, metrics::labels::DatabaseQuery};

pub async fn start(global: Arc<GlobalState>) -> Result<()> {
	info!("Starting listening to chat messages");

	let inserter = Arc::new(Mutex::new(Vec::new()));

	let channels: Vec<String> = global
		.config
		.twitch
		.channels
		.iter()
		.map(|c| format!("#{}", c))
		.collect();

	let credentials = Credentials::anon();

	let mut client = Client::builder().credentials(credentials).connect().await?;
	client.join_all(&channels).await?;

	tokio::spawn(commit_logs(Arc::clone(&inserter), Arc::clone(&global)));

	process_messages(client, inserter, global, channels).await
}

async fn commit_logs(inserter: Arc<Mutex<Vec<Log>>>, global: Arc<GlobalState>) {
	loop {
		tokio::time::sleep(Duration::from_secs(10)).await;
		let start = Instant::now();

		let logs = inserter.lock().await.drain(..).collect::<Vec<_>>();

		if !logs.is_empty() {
			let mut batch = global.db.insert("logs").unwrap();

			for log in &logs {
				batch.write(log).await.unwrap();
			}

			batch.end().await.unwrap();
		}

		global.metrics.observe_database_query(
			DatabaseQuery {
				query: "chat_batch_insert".into(),
			},
			start.elapsed().as_millis(),
		);

		info!(
			"Inserted {} logs in {}ms",
			logs.len(),
			start.elapsed().as_millis()
		);
	}
}

async fn process_messages(
	mut client: Client,
	inserter: Arc<Mutex<Vec<Log>>>,
	global: Arc<GlobalState>,
	channels: Vec<String>,
) -> Result<()> {
	while let Ok(msg) = client.recv().await {
		match msg.as_typed()? {
			Message::Privmsg(msg) if !global.ignored_users.contains(msg.sender().login()) => {
				let color = msg.color().map(ToString::to_string);
				let badges: Vec<String> = msg
					.badges()
					.map(|b| b.as_badge_data().name().to_string())
					.collect();

				inserter.lock().await.push(Log {
					channel: msg.channel()[1..].to_lowercase(),
					content: Some(msg.text().to_owned()),
					user_id: Some(msg.sender().id().to_owned()),
					username: msg.sender().login().to_lowercase(),
					log_type: LogType::Chat,
					created_at: time::OffsetDateTime::now_utc(),
					color,
					badges,
				});
			}
			Message::ClearChat(msg) if msg.action().is_ban() || msg.action().is_time_out() => {
				let (user_id, username) = match msg.action() {
					Action::Ban(ban) => (ban.id(), ban.user()),
					Action::TimeOut(timeout) => (timeout.id(), timeout.user()),
					_ => continue,
				};

				inserter.lock().await.push(Log {
					channel: msg.channel()[1..].to_lowercase(),
					content: None,
					user_id: Some(user_id.to_owned()),
					username: username.to_lowercase(),
					log_type: LogType::Ban,
					created_at: time::OffsetDateTime::now_utc(),
					badges: vec![],
					color: None,
				});
			}
			Message::Reconnect => {
				info!("Reconnecting to Twitch IRC");
				client.reconnect().await?;
				client.join_all(&channels).await?;
			}
			Message::Ping(ping) => client.pong(&ping).await?,
			_ => {}
		}
	}
	Ok(())
}
