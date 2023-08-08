use actix_web::web;
use global::GlobalState;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod database;
mod global;
mod twitch;
mod utils;

#[tokio::main]
async fn main() {
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

	info!("Loading environment variables");

	if env == "development" {
		dotenv::dotenv().ok();
	}

	let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	info!("Connecting to database at {}", database_url);

	let db = sqlx::postgres::PgPool::connect(&database_url)
		.await
		.unwrap();

	sqlx::migrate!("./migrations").run(&db).await.unwrap();

	let global = web::Data::new(GlobalState::new(db));
	let global_http = web::Data::clone(&global);

	info!("Starting main processes");

	tokio::spawn(twitch::chat::start(global));
	api::start(global_http).await.unwrap();
}
