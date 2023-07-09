use actix_web::web;
use global::GlobalState;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod api;
mod database;
mod global;
mod twitch;
mod utils;

#[tokio::main]
async fn main() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::DEBUG)
		.finish();

	tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

	let url = std::env::var("DATABASE_URL")
		.unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/logging".to_string());
	let db = sqlx::postgres::PgPool::connect(&url).await.unwrap();

	sqlx::migrate!("./migrations").run(&db).await.unwrap();

	let data = web::Data::new(GlobalState { db });
	let data_http = web::Data::clone(&data);

	tokio::spawn(twitch::chat::start(data));
	api::start(data_http).await.unwrap();
}
