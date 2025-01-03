use std::{io::Error, sync::Arc};

use anyhow::Result;
use poem::{
	get, handler,
	listener::TcpListener,
	middleware::{AddData, Cors},
	EndpointExt, Route, Server,
};
use tracing::info;
use utils::metrics::PrometheusExporter;

use crate::{global::GlobalState, ENV};

#[handler]
fn health() -> String {
	"OK".to_string()
}

pub async fn server(global: Arc<GlobalState>) -> Result<(), Error> {
	let port = std::env::var("PORT")
		.map(|s| s.parse().unwrap_or(4002))
		.unwrap_or(4002);
	let hostname = match ENV.as_str() {
		"development" => "localhost",
		"production" => "0.0.0.0",
		_ => "0.0.0.0",
	};

	info!("Starting HTTP server on port {}", port);

	let cors = Cors::new();

	let app = Route::new()
		.at("/health", get(health))
		.nest(
			"/metrics",
			PrometheusExporter::new(Arc::clone(&global.metrics.registry)),
		)
		.with(AddData::new(global))
		.with(cors);

	let listener = TcpListener::bind(format!("{}:{}", hostname, port));
	Server::new(listener).run(app).await
}
