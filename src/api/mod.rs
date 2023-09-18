use actix_cors::Cors;
use actix_web::{
	web::{self, Data},
	App, HttpServer,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::global::GlobalState;

mod v1;

#[derive(Deserialize, Serialize)]
pub struct ApiPaginationQuery {
	pub offset: Option<i64>,
	pub limit: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct ApiPaginationResponse<T> {
	pub offset: i64,
	pub data: T,
}

pub async fn start(global: web::Data<GlobalState>) -> std::io::Result<()> {
	let port = std::env::var("PORT")
		.map(|s| s.parse().unwrap_or(4001))
		.unwrap_or(4001);

	info!("Starting API server on port {}", port);

	HttpServer::new(move || {
		let cors = Cors::default()
			.allow_any_header()
			.allow_any_method()
			.allow_any_origin();

		App::new()
			.app_data(Data::clone(&global))
			.wrap(TracingLogger::default())
			.wrap(cors)
			.service(
				web::scope("/api/v1")
					.service(v1::index)
					.service(v1::log::user_logs)
					.service(v1::log::channel_log)
					.service(v1::log::user_active_channels)
					.service(v1::log::search_logs)
					.service(v1::log::top_users)
					.service(v1::log::top_users_channel)
					.service(v1::log::top_channels)
					.service(v1::stats::stats_size)
					.service(v1::chart::rate_chart),
			)
	})
	.bind(("0.0.0.0", port))
	.unwrap()
	.run()
	.await
}
