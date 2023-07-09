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
		let channels = utils::parse_file("./lists/channels.txt");
		let ignored_users = utils::parse_file("./lists/ignored_users.txt");

		Self {
			db,
			channels,
			ignored_users: ignored_users.into_iter().collect(),
		}
	}
}
