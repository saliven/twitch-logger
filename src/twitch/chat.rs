use actix_web::web;
use tracing::info;
use twitch_irc::{
	login::StaticLoginCredentials,
	message::{ClearChatAction, ServerMessage},
	ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{database::log::Log, global::GlobalState};

pub async fn start(global: web::Data<GlobalState>) {
	info!("Starting listening to chat messages");

	let channels = global.channels.clone();

	let config = ClientConfig::default();

	let (mut incoming_messages, client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

	let join_handle = tokio::spawn(async move {
		let global = global.clone();

		while let Some(message) = incoming_messages.recv().await {
			match message {
				ServerMessage::Privmsg(msg) => {
					if !global.ignored_users.contains(&msg.sender.login) && msg.message_text.len() > 1 {
						Log::create(
							&global.db,
							&msg.sender.login,
							&msg.channel_login,
							Some(&msg.message_text),
							"chat",
						)
						.await
						.unwrap();
					}
				}
				ServerMessage::ClearChat(msg) => {
					if !global.ignored_users.contains(&msg.channel_login) {
						if let ClearChatAction::UserBanned {
							user_login,
							user_id: _,
						} = msg.action
						{
							Log::create(&global.db, &user_login, &msg.channel_login, None, "ban")
								.await
								.unwrap();
						}
					}
				}
				_ => {}
			}
		}
	});

	for channel in channels {
		client.join(channel.into()).unwrap();
	}

	join_handle.await.unwrap();
}
