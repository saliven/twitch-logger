use std::{io::Error, sync::Arc};

use crate::{
	global::GlobalState,
	metrics::{self},
};
use anyhow::Result;
use poem::{
	get, handler,
	listener::TcpListener,
	middleware::{AddData, Cors},
	EndpointExt, Route, Server,
};
use tracing::info;
use utils::metrics::PrometheusExporter;

mod v1;

#[handler]
fn health() -> String {
	"OK".to_string()
}

pub async fn start(global: Arc<GlobalState>) -> Result<(), Error> {
	let port = std::env::var("PORT")
		.map(|s| s.parse().unwrap_or(4001))
		.unwrap_or(4001);

	info!("Starting API server on port {}", port);

	let v1_route = v1::route();

	let cors = Cors::new();

	let app = Route::new()
		.at("/health", get(health))
		.nest("/v1", v1_route)
		.nest(
			"/metrics",
			PrometheusExporter::new(Arc::clone(&global.metrics.registry)),
		)
		.around(metrics::middleware::metrics)
		.with(AddData::new(global))
		.with(cors);

	let listener = TcpListener::bind(format!("localhost:{}", port));
	Server::new(listener).run(app).await
}
