use std::collections::HashSet;

use clickhouse::Client;
use config::Config;
use tracing::info;

use crate::metrics::Metrics;

pub mod config;

pub struct GlobalState {
	pub db: Client,
	pub config: Config,
	pub ignored_users: HashSet<String>,
	pub metrics: Metrics,
}

impl GlobalState {
	pub fn new(config: Config, db: Client) -> Self {
		let ignored_users = config
			.twitch
			.ignored
			.clone()
			.into_iter()
			.collect::<HashSet<String>>();

		info!("Loaded {} channels", &config.twitch.channels.len());
		info!("Loaded ignored users: {}", &ignored_users.len());

		let metrics = Metrics::new();

		Self {
			db,
			config,
			ignored_users,
			metrics,
		}
	}
}
