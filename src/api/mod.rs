use actix_web::{
	web::{self, Data},
	App, HttpServer,
};
use serde::Deserialize;
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::global::GlobalState;

mod v1;

#[derive(Deserialize)]
pub struct ApiPagination {
	pub offset: Option<i64>,
	pub limit: Option<i64>,
}

pub async fn start(global_data: Data<GlobalState>) -> std::io::Result<()> {
	info!("Starting API server");

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::clone(&global_data))
			.wrap(TracingLogger::default())
			.service(
				web::scope("/api/v1")
					.service(v1::log::user_logs)
					.service(v1::log::top_users)
					.service(v1::log::user_active_channels),
			)
	})
	.bind(("0.0.0.0", 4001))
	.unwrap()
	.run()
	.await
}
