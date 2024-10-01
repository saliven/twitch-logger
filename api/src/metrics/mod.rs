use std::sync::Arc;

use labels::{DatabaseQuery, HttpRequest, HttpResponse};
use prometheus_client::{
	metrics::{counter::Counter, family::Family, histogram::Histogram},
	registry::Registry,
};
use utils::metrics::BUCKETS;

pub mod labels;
pub mod middleware;

#[derive(Debug)]
pub struct Metrics {
	pub registry: Arc<Registry>,
	http_requests: Family<HttpRequest, Counter>,
	http_responses: Family<HttpResponse, Counter>,
	database_queries_duration: Family<DatabaseQuery, Histogram>,
	http_requests_duration: Family<HttpRequest, Histogram>,
}

impl Metrics {
	pub fn new() -> Self {
		let mut registry = Registry::default();

		let http_requests = Family::default();
		let http_responses = Family::default();

		let database_queries_duration =
			Family::<_, _>::new_with_constructor(|| Histogram::new(BUCKETS.iter().copied()));
		let http_requests_duration =
			Family::<_, _>::new_with_constructor(|| Histogram::new(BUCKETS.iter().copied()));

		registry.register(
			"http_requests",
			"The number of HTTP requests",
			http_requests.clone(),
		);

		registry.register(
			"http_responses",
			"The number of HTTP responses",
			http_responses.clone(),
		);

		registry.register(
			"database_queries_duartion",
			"The duration of database queries",
			database_queries_duration.clone(),
		);

		registry.register(
			"http_request_duration",
			"The duration of HTTP requests",
			http_requests_duration.clone(),
		);

		Self {
			registry: Arc::new(registry),
			http_requests,
			http_responses,
			database_queries_duration,
			http_requests_duration,
		}
	}

	pub fn increase_http_requests(&self, request: HttpRequest) {
		self.http_requests.get_or_create(&request).inc();
	}

	pub fn increase_http_responses(&self, response: HttpResponse) {
		self.http_responses.get_or_create(&response).inc();
	}

	pub fn observe_database_query(&self, query: DatabaseQuery, duration: u128) {
		self
			.database_queries_duration
			.get_or_create(&query)
			.observe(duration as f64);
	}

	pub fn observe_http_request(&self, request: HttpRequest, duration: u128) {
		self
			.http_requests_duration
			.get_or_create(&request)
			.observe(duration as f64);
	}
}
