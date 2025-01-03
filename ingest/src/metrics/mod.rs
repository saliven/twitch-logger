use std::sync::Arc;

use labels::DatabaseQuery;
use prometheus_client::{
	metrics::{family::Family, histogram::Histogram},
	registry::Registry,
};
use utils::metrics::BUCKETS;

pub mod labels;

#[derive(Debug)]
pub struct Metrics {
	pub registry: Arc<Registry>,
	database_queries_duration: Family<DatabaseQuery, Histogram>,
}

impl Metrics {
	pub fn new() -> Self {
		let mut registry = Registry::default();

		let database_queries_duration =
			Family::<_, _>::new_with_constructor(|| Histogram::new(BUCKETS.iter().copied()));

		registry.register(
			"database_queries_duartion",
			"The duration of database queries",
			database_queries_duration.clone(),
		);

		Self {
			registry: Arc::new(registry),
			database_queries_duration,
		}
	}

	pub fn observe_database_query(&self, query: DatabaseQuery, duration: u128) {
		self
			.database_queries_duration
			.get_or_create(&query)
			.observe(duration as f64);
	}
}
