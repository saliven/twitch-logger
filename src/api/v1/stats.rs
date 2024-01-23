use axum::{extract::State, http::StatusCode, Json};

use crate::{
	database::stats::{get_size, Stats},
	global::GlobalState,
};

pub async fn stats_size(State(state): State<GlobalState>) -> (StatusCode, Json<Stats>) {
	let stats = get_size(&state.db).await.unwrap();

	(StatusCode::OK, Json(stats))
}
