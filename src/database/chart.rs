use cached::proc_macro::cached;
use serde::Serialize;

#[derive(Debug, Clone, Default, sqlx::FromRow, Serialize)]
pub struct Data {
	hour: chrono::DateTime<chrono::Utc>,
	count: i64,
}

#[cached(
	time = 600,
	time_refresh = true,
	result = true,
	key = "bool",
	convert = r#"{ true }"#
)]
pub async fn get_rate_chart(db: &sqlx::PgPool) -> Result<Vec<Data>, sqlx::Error> {
	let rows = sqlx::query_as::<_, Data>(
		"
      WITH hours AS (
        SELECT
          generate_series(date_trunc('hour',
              NOW() - interval '7 days'),
            date_trunc('hour',
              NOW()),
            interval '1 hour') AS hour_interval
      )
      SELECT
        hours.hour_interval as hour,
        COUNT(logs.*) AS count
      FROM
        hours
        LEFT JOIN logs ON logs.created_at >= hours.hour_interval
          AND logs.created_at < hours.hour_interval + interval '1 hour'
        GROUP BY
          hours.hour_interval
        ORDER BY
          hours.hour_interval;
    ",
	)
	.fetch_all(db)
	.await?;

	Ok(rows)
}
