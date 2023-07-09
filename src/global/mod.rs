#[derive(Clone)]
pub struct GlobalState {
	pub db: sqlx::PgPool,
}
