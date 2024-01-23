use cached::proc_macro::cached;
use chrono::{serde::ts_seconds_option, DateTime, Utc};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use sqlx::{prelude::*, FromRow};
use sqlx::{Postgres, QueryBuilder};
extern crate serde_json;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize, Default)]
#[sqlx(type_name = "log_type", rename_all = "lowercase")]
pub enum LogType {
	#[default]
	Chat,
	Ban,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize)]
pub struct Log {
	pub id: uuid::Uuid,
	pub user_id: Option<String>,
	pub username: String,
	pub channel: String,
	pub content: Option<String>,
	pub log_type: LogType,
	#[serde(with = "ts_seconds_option")]
	pub created_at: Option<DateTime<Utc>>,
}

impl Default for Log {
	fn default() -> Self {
		Self {
			id: uuid::Uuid::new_v4(),
			user_id: Some(String::new()),
			username: String::new(),
			channel: String::new(),
			content: None,
			log_type: LogType::Chat,
			created_at: Some(Utc::now()),
		}
	}
}

impl Log {
	pub async fn bulk_insert(db: &sqlx::PgPool, logs: Vec<Self>) -> Result<(), sqlx::Error> {
		let mut query_builder: QueryBuilder<Postgres> =
			QueryBuilder::new("INSERT INTO logs(id, user_id, username, content, log_type, channel) ");

		query_builder.push_values(logs, |mut b, log| {
			b.push_bind(log.id)
				.push_bind(log.user_id)
				.push_bind(log.username)
				.push_bind(log.content)
				.push_bind(log.log_type)
				.push_bind(log.channel);
		});

		let query = query_builder.build();

		query.execute(db).await?;

		Ok(())
	}

	pub async fn get_by_username(
		db: &sqlx::PgPool,
		username: &str,
		channel: &str,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, sqlx::Error> {
		let logs =
			sqlx::query_as::<_, Self>("SELECT id, username, channel, content, log_type, created_at, user_id FROM logs WHERE username = $1 AND channel = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
			.bind(username)
			.bind(channel)
			.bind(limit)
			.bind(offset)
			.fetch_all(db)
			.await?;

		Ok(logs)
	}

	pub async fn get_by_channel(
		db: &sqlx::PgPool,
		id: &str,
		channel: &str,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, sqlx::Error> {
		let uuid = uuid::Uuid::parse_str(id).unwrap();

		let logs =
			sqlx::query_as::<_, Self>("SELECT id, username, channel, content, log_type, created_at, user_id FROM logs WHERE created_at <= (SELECT created_at FROM logs WHERE id = $1) AND channel = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
			.bind(uuid)
			.bind(channel)
			.bind(limit)
			.bind(offset)
			.fetch_all(db)
			.await?;

		Ok(logs)
	}
}

#[cached(
	time = 3600,
	time_refresh = true,
	result = true,
	key = "bool",
	convert = r#"{ true }"#
)]
pub async fn get_top_users(db: &sqlx::PgPool) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let top_users = sqlx::query_as::<_, (String, i64)>(
		"SELECT username, COUNT(*) FROM logs GROUP BY username ORDER BY COUNT(*) DESC LIMIT 10",
	)
	.fetch_all(db)
	.await?;

	Ok(top_users)
}

pub async fn get_active_channels(
	db: &sqlx::PgPool,
	username: &str,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let channels =
		sqlx::query_as::<_, (String, i64)>("SELECT channel, count(id) as count FROM logs WHERE username = $1 GROUP BY channel ORDER BY count DESC")
		.bind(username)
		.fetch_all(db)
		.await?;

	Ok(channels)
}

#[cached(
	time = 3600,
	result = true,
	key = "String",
	convert = r#"{ String::from(channel) }"#
)]
pub async fn get_top_users_channel(
	db: &sqlx::PgPool,
	channel: &str,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let top_users =
		sqlx::query_as::<_, (String, i64)>("SELECT username, COUNT(*) FROM logs WHERE channel = $1 GROUP BY username ORDER BY COUNT(*) DESC LIMIT 10")
		.bind(channel)
		.fetch_all(db)
		.await?;

	Ok(top_users)
}

#[cached(
	time = 3600,
	time_refresh = true,
	result = true,
	key = "bool",
	convert = r#"{ true }"#
)]
pub async fn get_top_channels(db: &sqlx::PgPool) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let channels = sqlx::query_as::<_, (String, i64)>(
		"SELECT channel, count(id) as count FROM logs GROUP BY channel ORDER BY count DESC",
	)
	.fetch_all(db)
	.await?;

	Ok(channels)
}

#[cached(
	time = 3600,
	result = true,
	key = "String",
	convert = r#"{ String::from(query) }"#
)]
pub async fn search_users(db: &sqlx::PgPool, query: &str) -> Result<Vec<String>, sqlx::Error> {
	let users = sqlx::query_as::<_, (String,)>(
		"SELECT DISTINCT username FROM logs WHERE username ILIKE $1 LIMIT 10",
	)
	.bind(format!("%{}%", query))
	.fetch_all(db)
	.await?;

	Ok(users.into_iter().map(|(username,)| username).collect())
}
