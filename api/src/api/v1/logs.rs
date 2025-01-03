use std::sync::Arc;

use clickhouse::Row;
use poem::web::Data;
use poem_openapi::{
	param::Query,
	payload::{Json, PlainText},
	types::{ParseFromJSON, ToJSON},
	ApiResponse, Object, OpenApi,
};
use serde::{Deserialize, Serialize};
use utils::{
	database::{self, Log},
	filter::{FieldDefinition, FieldType, Filter},
};

use crate::{global::GlobalState, metrics::labels::DatabaseQuery};

pub struct Logs;

#[derive(Debug, ApiResponse)]
pub enum LogResponse<T: Send + Sync + ToJSON> {
	#[oai(status = 200)]
	Ok(Json<T>),
	#[oai(status = 404)]
	NotFound,
	#[oai(status = 500)]
	InternalServerError,
}

#[derive(Debug, Object, Clone, Serialize, Deserialize, Row)]
pub struct StatsObject {
	total_rows: i32,
	total_bytes: i64,
}

#[derive(Debug, Object)]
pub struct KeyValueObject<T: ParseFromJSON + ToJSON> {
	pub key: String,
	pub value: T,
}

impl<T: ParseFromJSON + ToJSON> KeyValueObject<T> {
	pub fn from_vec(vec: Vec<(String, T)>) -> Vec<Self> {
		vec
			.into_iter()
			.map(|(key, value)| KeyValueObject { key, value })
			.collect()
	}
}

#[OpenApi(prefix_path = "/logs")]
impl Logs {
	#[oai(path = "/", method = "get")]
	pub async fn logs(
		&self,
		state: Data<&Arc<GlobalState>>,
		cursor: Query<Option<String>>,
		filter: Query<Option<String>>,
		#[oai(validator(maximum(value = "100")))] limit: Query<Option<u32>>,
	) -> LogResponse<utils::pagination::PaginatedResponse<Log>> {
		// TODO: when there are multiple filters with the same field, it should be combined with OR
		let state = state.0;
		let field_definitions = vec![
			FieldDefinition {
				name: "username".to_string(),
				field_type: FieldType::String,
			},
			FieldDefinition {
				name: "channel".to_string(),
				field_type: FieldType::String,
			},
			FieldDefinition {
				name: "log_type".to_string(),
				field_type: FieldType::Integer,
			},
			FieldDefinition {
				name: "created_at".to_string(),
				field_type: FieldType::DateTime,
			},
			FieldDefinition {
				name: "badges".to_string(),
				field_type: FieldType::Array(Box::new(FieldType::String)),
			},
		];

		let filter = filter
			.0
			.map(|filter| Filter::new(filter, &field_definitions).unwrap());

		let start = std::time::Instant::now();
		let results = utils::pagination::paginate(cursor.0, limit.0, move |cursor, limit| {
			let state = state.clone();
			let filter = filter.clone();

			async move {
				database::logs(&state.db, filter, cursor, limit)
					.await
					.map_err(|e| anyhow::anyhow!(e))
			}
		})
		.await;

		state.metrics.observe_database_query(
			DatabaseQuery {
				query: "logs".into(),
			},
			start.elapsed().as_millis(),
		);

		LogResponse::Ok(results.unwrap())
	}

	#[oai(path = "/filter_suggestions", method = "get")]
	pub async fn filter_suggestions(
		&self,
		state: Data<&Arc<GlobalState>>,
		filter: Query<Option<String>>,
	) -> PlainText<String> {
		// TODO: suggest filter fields based on the filter string
		// - e.g. if the filter string is "username:foo", suggest "channel" and "log_type" based on the most active channels and log types
		// - e.g. if the filter string is "channel:foo", suggest "username" and "log_type" based on the most active users and log types
		// - e.g. if the filter string is "log_type:foo", suggest "username" and "channel" based on the most active users and channels

		PlainText("Test".into())
	}

	#[oai(path = "/user/search", method = "get")]
	pub async fn search_users(
		&self,
		state: Data<&Arc<GlobalState>>,
		username: Query<String>,
	) -> LogResponse<Vec<String>> {
		let users = database::search_users(&state.db, &username).await;

		match users {
			Ok(users) => LogResponse::Ok(Json(users)),
			Err(_) => LogResponse::InternalServerError,
		}
	}

	#[oai(path = "/users/top", method = "get")]
	pub async fn top_users(
		&self,
		state: Data<&Arc<GlobalState>>,
	) -> LogResponse<Vec<KeyValueObject<i64>>> {
		let users = database::get_top_users(&state.db).await;

		match users {
			Ok(users) => LogResponse::Ok(Json(KeyValueObject::from_vec(users))),
			Err(_) => LogResponse::InternalServerError,
		}
	}

	#[oai(path = "/channels/top", method = "get")]
	pub async fn top_channels(
		&self,
		state: Data<&Arc<GlobalState>>,
	) -> LogResponse<Vec<KeyValueObject<i64>>> {
		let channels = database::get_top_channels(&state.db).await;

		match channels {
			Ok(channels) => LogResponse::Ok(Json(KeyValueObject::from_vec(channels))),
			Err(_) => LogResponse::InternalServerError,
		}
	}

	#[oai(path = "/stats", method = "get")]
	pub async fn stats(&self, state: Data<&Arc<GlobalState>>) -> LogResponse<StatsObject> {
		let count = database::get_log_count(&state.db);
		let size = database::get_size(&state.db);
		let (count, size) = tokio::join!(count, size);

		LogResponse::Ok(Json(StatsObject {
			total_bytes: size.unwrap(),
			total_rows: count.unwrap(),
		}))
	}
}
