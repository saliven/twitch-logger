use chrono::{serde::ts_seconds_option, DateTime, Utc};
use serde::Serialize;
#[allow(unused_imports)]
use sqlx::{prelude::*, FromRow};
extern crate serde_json;

#[derive(Debug, Clone, Default, sqlx::FromRow, Serialize)]
pub struct Log {
	pub id: uuid::Uuid,
	pub username: String,
	pub channel: String,
	pub content: Option<String>,
	pub log_type: String,
	#[serde(with = "ts_seconds_option")]
	pub created_at: Option<DateTime<Utc>>,
}

impl Log {
	pub async fn create(
		db: &sqlx::PgPool,
		username: &str,
		channel: &str,
		content: Option<&str>,
		log_type: &str,
	) -> Result<Self, sqlx::Error> {
		let id = uuid::Uuid::new_v4();
		let created_at = chrono::Utc::now();

		sqlx::query(
			"INSERT INTO logs (id, username, content, log_type, created_at, channel) VALUES ($1, $2, $3, $4, $5, $6)",
		)
		.bind(&id)
		.bind(username)
		.bind(content)
		.bind(log_type)
		.bind(&created_at)
		.bind(channel)
		.execute(db)
		.await?;

		Ok(Self {
			id,
			username: username.to_string(),
			channel: channel.to_string(),
			content: content.map(|s| s.to_string()),
			log_type: log_type.to_string(),
			created_at: Some(created_at),
		})
	}

	pub async fn get_by_username(
		db: &sqlx::PgPool,
		username: &str,
		channel: &str,
		limit: i64,
		offset: i64,
	) -> Result<Vec<Self>, sqlx::Error> {
		let logs = 
			sqlx::query_as::<_, Self>("SELECT id, username, channel, content, log_type, created_at FROM logs WHERE username = $1 AND channel = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4")
			.bind(username)
			.bind(channel)
			.bind(limit)
			.bind(offset)
			.fetch_all(db)
			.await?;

		Ok(logs)
	}

}

pub async fn get_top_users(db: &sqlx::PgPool) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let top_users = 
		sqlx::query_as::<_, (String, i64)>("SELECT username, COUNT(*) FROM logs GROUP BY username ORDER BY COUNT(*) DESC LIMIT 10")
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

pub async fn get_top_channels(db: &sqlx::PgPool) -> Result<Vec<(String, i64)>, sqlx::Error> {
	let channels = 
		sqlx::query_as::<_, (String, i64)>("SELECT channel, count(id) as count FROM logs GROUP BY channel ORDER BY count DESC")
		.fetch_all(db)
		.await?;

	Ok(channels)
}
