use axum::{extract::State, http::StatusCode, Json};

use crate::{
	database::chart::{self, Data},
	global::GlobalState,
};

pub async fn rate_chart(State(state): State<GlobalState>) -> (StatusCode, Json<Vec<Data>>) {
	let data = chart::get_rate_chart(&state.db).await.unwrap();

	(StatusCode::OK, Json(data))
}
