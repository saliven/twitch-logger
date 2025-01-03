use anyhow::Result;
use lazy_static::lazy_static;
use tokio::select;
use tracing::{info, Level};
use tracing_subscriber::fmt;

lazy_static! {
	pub static ref ENV: String =
		std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());
}

mod global;
mod http;
mod metrics;
mod twitch;

#[tokio::main]
async fn main() -> Result<()> {
	let fallback_log_level = match ENV.as_str() {
		"development" => Level::DEBUG,
		"production" => Level::INFO,
		_ => Level::DEBUG,
	};

	let filter_level = std::env::var("RUST_LOG")
		.map(|s| s.parse().unwrap_or(fallback_log_level))
		.unwrap_or(fallback_log_level);

	match ENV.as_str() {
		"production" => fmt()
			.with_max_level(filter_level)
			.json()
			.flatten_event(true)
			.with_target(false)
			.with_thread_ids(true)
			.with_file(true)
			.with_line_number(true)
			.init(),
		_ => fmt().with_max_level(filter_level).compact().init(),
	};

	info!("Starting up");

	let config = global::config::load();

	let db = clickhouse::Client::default()
		.with_url(&config.clickhouse.url)
		.with_password(&config.clickhouse.password)
		.with_user(&config.clickhouse.user)
		.with_database(&config.clickhouse.database);

	let global = std::sync::Arc::new(global::GlobalState::new(config, db));

	let twitch_future = tokio::spawn(twitch::start(global.clone()));
	let http_future = tokio::spawn(http::server(global.clone()));
	let ctrl_c_future = tokio::spawn(tokio::signal::ctrl_c());

	select! {
		e = twitch_future => anyhow::bail!("Twitch future exited! {:?}", e),
		e = http_future => anyhow::bail!("HTTP future exited! {:?}", e),
		_ = ctrl_c_future => info!("Shutting down!"),
	}

	Ok(())
}
