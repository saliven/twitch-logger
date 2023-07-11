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

pub async fn start(global_data: Data<GlobalState>) -> std::io::Result<()> {
	info!("Starting API server");

	HttpServer::new(move || {
		let cors = Cors::default()
			.allow_any_header()
			.allow_any_method()
			.allow_any_origin();

		App::new()
			.app_data(web::Data::clone(&global_data))
			.wrap(TracingLogger::default())
			.wrap(cors)
			.service(
				web::scope("/api/v1")
					.service(v1::index)
					.service(v1::log::user_logs)
					.service(v1::log::user_active_channels)
					.service(v1::log::search_logs)
					.service(v1::log::top_users)
					.service(v1::log::top_users_channel)
					.service(v1::log::top_channels)
					.service(v1::stats::stats_size),
			)
	})
	.bind(("0.0.0.0", 4001))
	.unwrap()
	.run()
	.await
}
