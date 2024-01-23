use anyhow::Result;
use global::GlobalState;
use tokio::select;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod database;
mod global;
mod twitch;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
	let env = std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string());

	let fallback_log_level = match env.as_str() {
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

	if env == "development" {
		info!("Loading environment variables");
		dotenv::dotenv().ok();
	}

	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	info!("Connecting to database at {}", database_url);

	let db = sqlx::postgres::PgPool::connect(&database_url)
		.await
		.unwrap();

	sqlx::migrate!("./migrations").run(&db).await.unwrap();

	let global = GlobalState::new(db);

	let chat_future = tokio::spawn(twitch::chat::start(global.clone()));
	let api_future = tokio::spawn(api::start(global.clone()));
	let ctrl_c_future = tokio::spawn(tokio::signal::ctrl_c());

	select! {
		_ = chat_future => error!("Chat future exited!"),
		_ = api_future => error!("Api future existed!"),
		_ = ctrl_c_future => error!("Ctrl+C Signal received! Terminating...")
	}

	Ok(())
}
