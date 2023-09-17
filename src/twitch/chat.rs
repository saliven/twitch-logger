use actix_web::web;
use chrono::Utc;
use tokio::{sync::Mutex, time::Instant};
use tracing::{debug, error, info};
use twitch_irc::{
	login::StaticLoginCredentials,
	message::{ClearChatAction, ServerMessage},
	ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{database::log::Log, global::GlobalState};

static THRESHOLD: usize = 200;
static TIME_THRESHOLD: u64 = 20;

pub async fn start(global: web::Data<GlobalState>) {
	info!("Starting listening to chat messages");

	let logs = Mutex::new(Vec::new());
	let mut last_flush = Instant::now();

	let channels = global.channels.clone();

	let config = ClientConfig::default();

	let (mut incoming_messages, client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

	let join_handle = tokio::spawn(async move {
		let global = global.clone();

		while let Some(message) = incoming_messages.recv().await {
			let mut logs_vec = logs.lock().await;

			match message {
				ServerMessage::Privmsg(msg) => {
					if !global.ignored_users.contains(&msg.sender.login) && msg.message_text.len() > 1 {
						logs_vec.push(Log {
							id: uuid::Uuid::new_v4(),
							username: msg.sender.login,
							channel: msg.channel_login,
							content: Some(msg.message_text),
							log_type: "chat".into(),
							created_at: Some(Utc::now()),
						});
					}
				}
				ServerMessage::ClearChat(msg) => {
					if !global.ignored_users.contains(&msg.channel_login) {
						if let ClearChatAction::UserBanned {
							user_login,
							user_id: _,
						}
						| ClearChatAction::UserTimedOut {
							user_login,
							user_id: _,
							timeout_length: _,
						} = msg.action
						{
							logs_vec.push(Log {
								id: uuid::Uuid::new_v4(),
								username: user_login,
								channel: msg.channel_login,
								content: None,
								log_type: "ban".into(),
								created_at: Some(Utc::now()),
							});
						}
					}
				}
				_ => {}
			}

			if logs_vec.len() >= THRESHOLD || last_flush.elapsed().as_secs() >= TIME_THRESHOLD {
				let len = logs_vec.len();

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
	});

	for channel in channels {
		client.join(channel.into()).unwrap();
	}

	join_handle.await.unwrap();
}
