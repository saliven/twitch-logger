use axum::{
	extract::{Path, Query, State},
	http::StatusCode,
	Json,
};
use serde::Deserialize;

use crate::{
	api::{ApiPaginationQuery, ApiPaginationResponse},
	database::log::{self, Log},
	global::GlobalState,
};

pub async fn user_logs(
	State(state): State<GlobalState>,
	Path((username, channel)): Path<(String, String)>,
	Query(query): Query<ApiPaginationQuery>,
) -> (StatusCode, Json<ApiPaginationResponse<Vec<Log>>>) {
	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_username(
		&state.db,
		&username.to_lowercase(),
		&channel.to_lowercase(),
		limit,
		offset,
	)
	.await
	.unwrap();

	(
		StatusCode::OK,
		Json(ApiPaginationResponse { data: logs, offset }),
	)
}

#[derive(Deserialize)]
pub struct ChannelLogsQuery {
	message_id: String,
	offset: Option<i64>,
	limit: Option<i64>,
}

pub async fn channel_log(
	State(state): State<GlobalState>,
	Path(channel): Path<String>,
	Query(query): Query<ChannelLogsQuery>,
) -> (StatusCode, Json<ApiPaginationResponse<Vec<Log>>>) {
	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_channel(
		&state.db,
		&query.message_id,
		&channel.to_lowercase(),
		limit,
		offset,
	)
	.await
	.unwrap();

	(
		StatusCode::OK,
		Json(ApiPaginationResponse { data: logs, offset }),
	)
}

#[derive(Deserialize)]
pub struct SearchQuery {
	query: String,
}

pub async fn search_logs(
	State(state): State<GlobalState>,
	Query(query): Query<SearchQuery>,
) -> (StatusCode, Json<Vec<String>>) {
	let users = log::search_users(&state.db, query.query.as_str())
		.await
		.unwrap();

	(StatusCode::OK, Json(users))
}

pub async fn user_active_channels(
	State(state): State<GlobalState>,
	Path(username): Path<String>,
) -> (StatusCode, Json<Vec<(String, i64)>>) {
	let channels = log::get_active_channels(&state.db, &username.to_lowercase())
		.await
		.unwrap();

	(StatusCode::OK, Json(channels))
}

pub async fn top_users(State(state): State<GlobalState>) -> (StatusCode, Json<Vec<(String, i64)>>) {
	let users = log::get_top_users(&state.db).await.unwrap();

	(StatusCode::OK, Json(users))
}

pub async fn top_users_channel(
	State(state): State<GlobalState>,
	Path(channel): Path<String>,
) -> (StatusCode, Json<Vec<(String, i64)>>) {
	let users = log::get_top_users_channel(&state.db, &channel.to_lowercase())
		.await
		.unwrap();

	(StatusCode::OK, Json(users))
}

pub async fn top_channels(
	State(state): State<GlobalState>,
) -> (StatusCode, Json<Vec<(String, i64)>>) {
	let channels = log::get_top_channels(&state.db).await.unwrap();

	(StatusCode::OK, Json(channels))
}

pub async fn username_history(
	State(state): State<GlobalState>,
	Path(username): Path<String>,
) -> (StatusCode, Json<Vec<String>>) {
	let history = log::username_history(&state.db, &username.to_lowercase())
		.await
		.unwrap();

	(StatusCode::OK, Json(history))
}
