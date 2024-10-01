use std::sync::Arc;

use poem::{Endpoint, IntoResponse, Request, Response, Result};

use crate::global::GlobalState;

use super::labels::{HttpMethod, HttpRequest, HttpResponse};

pub async fn metrics<E: Endpoint>(next: E, req: Request) -> Result<Response> {
	let data = req.data::<Arc<GlobalState>>().unwrap().clone();

	let metrics_request = HttpRequest {
		path: req.uri().path().to_string(),
		method: HttpMethod::from(req.method().clone()),
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

			Ok(resp)
		}
		Err(err) => {
			data.metrics.increase_http_responses(HttpResponse {
				status_code: err.status().as_u16(),
			});

			Err(err)
		}
	}
}
