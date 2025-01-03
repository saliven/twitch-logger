use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::Result;
use cached::proc_macro::cached;
use clickhouse::{Client, Row};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use sql_builder::SqlBuilder;
use time::OffsetDateTime;

use crate::{
	filter::Filter,
	pagination::{decode_cursor, Pageable},
};

#[derive(Debug, Clone, Default, Serialize_repr, Deserialize_repr, Enum)]
#[oai(rename_all = "camelCase")]
#[repr(i8)]
pub enum LogType {
	#[default]
	Chat = 1,
	Ban = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize, Row, Object)]
pub struct Log {
	pub username: String,
	pub channel: String,
	pub content: Option<String>,
	pub log_type: LogType,
	#[serde(with = "clickhouse::serde::time::datetime64::nanos")]
	pub created_at: OffsetDateTime,
	pub user_id: Option<String>,
	pub color: Option<String>,
	pub badges: Vec<String>,
}

impl Pageable for Log {
	fn get_cursor_values(&self) -> HashMap<String, String> {
		let mut values = HashMap::new();
		values.insert(
			"created_at".to_string(),
			self.created_at.unix_timestamp().to_string(),
		);
		values
	}
}

#[cached(
	time = 10,
	result = true,
	convert = r#"{ logs_cache_key(&filters, &cursor, limit) }"#,
	key = "String"
)]
pub async fn logs(
	db: &Client,
	filters: Option<Filter>,
	cursor: Option<String>,
	limit: u32,
) -> Result<Vec<Log>> {
	let mut builder = SqlBuilder::select_from("logs");
	builder.field("?fields");

	if let Some(filters) = filters {
		filters.apply_to_query_builder(&mut builder);
	}

	if let Some(cursor_str) = cursor {
		let cursor_values = decode_cursor(&cursor_str);
		if let Some(created_at) = cursor_values.get("created_at") {
			builder.and_where_lt("created_at", created_at);
		}
	}

	let mut sql = builder.order_desc("created_at").limit(limit).sql()?;
	sql.pop();

	let db_query = db.query(&sql);

	let logs = db_query.fetch_all().await?;

	tracing::info!(
		database.query = sql,
		database.length = logs.len(),
		"Fetched {} logs",
		logs.len()
	);

	Ok(logs)
}

fn logs_cache_key(filters: &Option<Filter>, cursor: &Option<String>, limit: u32) -> String {
	let mut hasher = DefaultHasher::new();

	if let Some(filters) = filters {
		filters.hash(&mut hasher);
	}
	if let Some(cursor) = cursor {
		cursor.hash(&mut hasher);
	}
	limit.hash(&mut hasher);

	hasher.finish().to_string()
}

#[cached(
	time = 3600,
	result = true,
	convert = r#"{ username.into() }"#,
	key = "String"
)]
pub async fn get_active_channels(
	db: &Client,
	username: &str,
) -> Result<Vec<(String, i64)>, clickhouse::error::Error> {
	db.query("SELECT channel, count() as count FROM logs WHERE username = ? GROUP BY channel ORDER BY count DESC")
	.bind(username)
	.fetch_all()
	.await
}

#[cached(time = 3600, result = true, convert = r#"{ true }"#, key = "bool")]
pub async fn get_top_users(db: &Client) -> Result<Vec<(String, i64)>, clickhouse::error::Error> {
	db.query("SELECT username, count() FROM logs WHERE log_type = 1 GROUP BY username ORDER BY count() DESC LIMIT 10")
		.fetch_all()
		.await
}

#[cached(
	time = 3600,
	result = true,
	convert = r#"{ channel.into() }"#,
	key = "String"
)]
pub async fn get_top_users_channel(
	db: &Client,
	channel: &str,
) -> Result<Vec<(String, i64)>, clickhouse::error::Error> {
	db.query("SELECT username, count() FROM logs WHERE channel = ? GROUP BY username ORDER BY count() DESC LIMIT 10")
		.bind(channel)
		.fetch_all()
		.await
}

#[cached(time = 3600, result = true, convert = r#"{ true }"#, key = "bool")]
pub async fn get_top_channels(db: &Client) -> Result<Vec<(String, i64)>, clickhouse::error::Error> {
	db.query("SELECT channel, count(id) as count FROM logs GROUP BY channel ORDER BY count DESC")
		.fetch_all()
		.await
}

#[cached(
	time = 3600,
	result = true,
	convert = r#"{ username.into() }"#,
	key = "String"
)]
pub async fn search_users(
	db: &Client,
	username: &str,
) -> Result<Vec<String>, clickhouse::error::Error> {
	db.query("SELECT DISTINCT username FROM logs WHERE username ILIKE ? LIMIT 20")
		.bind(format!("%{}%", username))
		.fetch_all()
		.await
}

#[cached(time = 3600, result = true, convert = r#"{ true }"#, key = "bool")]
pub async fn get_log_count(db: &Client) -> Result<i32, clickhouse::error::Error> {
	db.query("SELECT count(*) FROM logs")
		.fetch_one::<i32>()
		.await
}

#[cached(time = 3600, result = true, convert = r#"{ true }"#, key = "bool")]
pub async fn get_size(db: &Client) -> Result<i64, clickhouse::error::Error> {
	db.query(
		"SELECT sum(bytes) as total_bytes
	 FROM cluster('clickhouse-cluster', system.parts)
	 WHERE database = 'logger' AND table = 'logs_local'",
	)
	.fetch_one::<i64>()
	.await
}
