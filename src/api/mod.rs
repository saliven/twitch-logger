use actix_web::{
	web::{self, Data},
	App, HttpServer,
};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::global::GlobalState;

mod v1;

pub async fn start(data: Data<GlobalState>) -> std::io::Result<()> {
	info!("Starting API server");

	HttpServer::new(move || {
		App::new()
			.app_data(web::Data::clone(&data))
			.wrap(TracingLogger::default())
			.service(web::scope("/api/v1").service(v1::log::user_logs))
		// .service()
	})
	.bind(("0.0.0.0", 4001))
	.unwrap()
	.run()
	.await
}
