use std::sync::Arc;

use anyhow::Result;
use clickhouse::Client;
use global::{config, GlobalState};
use lazy_static::lazy_static;
use metrics::Metrics;
use tokio::select;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod global;
mod metrics;

lazy_static! {
	pub static ref ENV: String =
		std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
}

#[tokio::main]
async fn main() -> Result<()> {
	let fallback_log_level = match ENV.as_str() {
		"development" => Level::DEBUG,
		"production" => Level::INFO,
		_ => Level::DEBUG,
	};

	let filter_level = std::env::var("RUST_LOG")
		.map(|s| s.parse().unwrap_or(fallback_log_level))
		.unwrap_or(Level::DEBUG);

	let subscriber = FmtSubscriber::builder()
		.with_max_level(filter_level)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

	info!("Starting up");

	let config = config::load();

	let db = Client::default()
		.with_url(&config.clickhouse.url)
		.with_password(&config.clickhouse.password)
		.with_user(&config.clickhouse.user)
		.with_database(&config.clickhouse.database);

	let metrics = Metrics::new();

	let global = Arc::new(GlobalState { db, metrics });

	let api_future = tokio::spawn(api::start(global.clone()));
	let ctrl_c_future = tokio::spawn(tokio::signal::ctrl_c());

	select! {
		e = api_future => anyhow::bail!("API future exited! {:?}", e),
		_ = ctrl_c_future => info!("Shutting down!")
	}

	Ok(())
}
