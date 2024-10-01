use std::{io::Error, sync::Arc};

use anyhow::Result;
use poem::{
	listener::TcpListener,
	middleware::{AddData, Cors},
	EndpointExt, Route, Server,
};
use tracing::info;
use utils::metrics::PrometheusExporter;

use crate::global::GlobalState;

pub async fn server(global: Arc<GlobalState>) -> Result<(), Error> {
	let port = std::env::var("PORT")
		.map(|s| s.parse().unwrap_or(4002))
		.unwrap_or(4002);

	info!("Starting metrics server on port {}", port);

	let cors = Cors::new();

	let app = Route::new()
		.nest(
			"/metrics",
			PrometheusExporter::new(Arc::clone(&global.metrics.registry)),
		)
		.with(AddData::new(global))
		.with(cors);

	let listener = TcpListener::bind(format!("localhost:{}", port));
	Server::new(listener).run(app).await
}
