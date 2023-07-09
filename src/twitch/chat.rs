use actix_web::web;
use twitch_irc::{
	login::StaticLoginCredentials,
	message::{ClearChatAction, ServerMessage},
	ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

use crate::{database::log::Log, global::GlobalState, utils};

pub async fn start(data: web::Data<GlobalState>) {
	let channels = utils::load_channels();

	let config = ClientConfig::default();

	let (mut incoming_messages, client) =
		TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

	let join_handle = tokio::spawn(async move {
		while let Some(message) = incoming_messages.recv().await {
			match message {
				ServerMessage::Privmsg(msg) => {
					Log::create(
						&data.db,
						&msg.sender.login,
						&msg.channel_login,
						Some(&msg.message_text),
						"chat",
					)
					.await
					.unwrap();
				}
				ServerMessage::ClearChat(msg) => {
					if let ClearChatAction::UserBanned {
						user_login,
						user_id: _,
					} = msg.action
					{
						Log::create(&data.db, &user_login, &msg.channel_login, None, "ban")
							.await
							.unwrap();
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
