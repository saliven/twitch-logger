use std::sync::Arc;

use poem::{Endpoint, IntoResponse, Request, Response, Result};
use tracing::{error, info};

use crate::global::GlobalState;

use super::labels::{HttpMethod, HttpRequest, HttpResponse};

pub async fn metrics<E: Endpoint>(next: E, req: Request) -> Result<Response> {
	let data = req.data::<Arc<GlobalState>>().unwrap().clone();

	let uri = req.uri().clone();

	let method = req.method().clone();
	let path = uri.path().to_string();
	let query = uri.query().unwrap_or_default();

	let metrics_request = HttpRequest {
		path: path.clone(),
		method: HttpMethod::from(method.clone()),
	};

	data.metrics.increase_http_requests(metrics_request.clone());

	let start = std::time::Instant::now();
	let res = next.call(req).await;

	data
		.metrics
		.observe_http_request(metrics_request, start.elapsed().as_millis());

	match res {
		Ok(resp) => {
			let resp = resp.into_response();

			data.metrics.increase_http_responses(HttpResponse {
				status_code: resp.status().as_u16(),
			});

			info!(
				http.method = method.as_str(),
				http.path = path,
				http.status_code = resp.status().as_u16(),
				http.response_time = start.elapsed().as_millis(),
				http.query = query,
				"HTTP request processed in {}ms",
				start.elapsed().as_millis()
			);

			Ok(resp)
		}
		Err(err) => {
			data.metrics.increase_http_responses(HttpResponse {
				status_code: err.status().as_u16(),
			});

			error!(
				http.method = method.as_str(),
				http.path = path,
				http.status_code = err.status().as_u16(),
				http.response_time = start.elapsed().as_millis(),
				http.query = query,
				"HTTP request failed after {}ms",
				start.elapsed().as_millis()
			);

			Err(err)
		}
	}
}
