use std::sync::Arc;

use poem::http::StatusCode;
use poem::{Endpoint, Request, Response, Result};
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use tracing::info;

pub const BUCKETS: [f64; 7] = [0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];

pub struct PrometheusExporter {
	pub registry: Arc<Registry>,
}

impl PrometheusExporter {
	pub fn new(registry: Arc<Registry>) -> Self {
		Self { registry }
	}
}

impl Endpoint for PrometheusExporter {
	type Output = Response;

	async fn call(&self, _req: Request) -> Result<Self::Output> {
		info!("Serving metrics");

		let mut buffer = String::new();
		encode(&mut buffer, &self.registry)
			.map(|()| {
				Response::builder()
					.content_type("text/plain; charset=utf-8")
					.body(buffer)
			})
			.map_err(|_err| StatusCode::INTERNAL_SERVER_ERROR.into())
	}
}
