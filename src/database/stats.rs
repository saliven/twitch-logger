use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Stats {
	total_count: i64,
	db_size: String,
}

impl Stats {
	pub async fn get_size(db: &sqlx::PgPool) -> Result<Self, sqlx::Error> {
		let total_count = sqlx::query_scalar("SELECT COUNT(*) FROM logs")
			.fetch_one(db)
			.await?;

		let db_size = sqlx::query_scalar("SELECT pg_size_pretty(pg_database_size(current_database()))")
			.fetch_one(db)
			.await?;

		Ok(Self {
			total_count,
			db_size,
		})
	}
}
