use clickhouse::Client;

use crate::metrics::Metrics;

pub mod config;

pub struct GlobalState {
	pub db: Client,
	pub metrics: Metrics,
}
