use std::collections::HashSet;

use crate::utils;

#[derive(Clone)]
pub struct GlobalState {
	pub db: sqlx::PgPool,
	pub channels: Vec<String>,
	pub ignored_users: HashSet<String>,
}

impl GlobalState {
	pub fn new(db: sqlx::PgPool) -> Self {
		let bots = utils::parse_file("./lists/bots.txt");

		let channels = utils::parse_env_list("TWITCH_CHANNELS");
		let ignored_usernames = utils::parse_env_list("TWITCH_IGNORED_USERS");

		let ignored_users = ignored_usernames
			.into_iter()
			.chain(bots.into_iter())
			.collect::<HashSet<String>>();

		Self {
			db,
			channels,
			ignored_users,
		}
	}
}
