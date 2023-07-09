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
	let filter_level = std::env::var("RUST_LOG")
		.map(|s| s.parse().unwrap_or(Level::DEBUG))
		.unwrap_or(Level::DEBUG);

	let subscriber = FmtSubscriber::builder()
		.with_max_level(filter_level)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

	let url = std::env::var("DATABASE_URL")
		.unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/logging".to_string());

	info!("Connecting to database at {}", url);

	let db = sqlx::postgres::PgPool::connect(&url).await.unwrap();

	sqlx::migrate!("./migrations").run(&db).await.unwrap();

	let data = web::Data::new(GlobalState { db });
	let data_http = web::Data::clone(&data);

	tokio::spawn(twitch::chat::start(data));
	api::start(data_http).await.unwrap();
}
