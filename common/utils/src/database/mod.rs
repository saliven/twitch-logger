use std::{
	collections::HashMap,
	hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::Result;
use builder::QueryBuilder;
use cached::proc_macro::cached;
use clickhouse::{Client, Row};
use poem_openapi::{Enum, Object};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use time::OffsetDateTime;

use crate::{
	filter::Filter,
	pagination::{decode_cursor, Pageable},
};

pub mod builder;

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
	#[serde(with = "clickhouse::serde::time::datetime64::micros")]
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

#[derive(Debug, Object, Clone, Serialize, Deserialize, Row)]
pub struct StatsObject {
	total_rows: i64,
	total_bytes: i64,
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
) -> Result<Vec<Log>, clickhouse::error::Error> {
	let mut builder = QueryBuilder::new().select(&["?fields"]).from("logs");

	if let Some(filters) = filters {
		builder = filters.apply_to_query_builder(builder);
	}

	if let Some(cursor_str) = cursor {
		let cursor_values = decode_cursor(&cursor_str);
		if let Some(created_at) = cursor_values.get("created_at") {
			builder = builder.where_clause(&[(
				"created_at".to_string(),
				"<".to_string(),
				created_at.to_string(),
			)]);
		}
	}

	builder = builder.order_by("created_at", "DESC").limit(limit as usize);

	let (query, params) = builder.build();

	let mut db_query = db.query(&query);

	for param in params {
		db_query = db_query.bind(param);
	}
	let logs = db_query.fetch_all().await?;

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
pub async fn get_stats(db: &Client) -> Result<StatsObject, clickhouse::error::Error> {
	db.query(
		"SELECT
    sum(rows) AS total_rows,
    sum(bytes) AS total_bytes
FROM system.parts
WHERE active AND database = 'logger' AND table = 'logs'
GROUP BY table
ORDER BY sum(bytes) DESC",
	)
	.fetch_one::<StatsObject>()
	.await
}
