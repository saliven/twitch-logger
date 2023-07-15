use actix_web::{get, http::Error, web, HttpResponse};
use serde::Deserialize;

use crate::{
	api::{ApiPaginationQuery, ApiPaginationResponse},
	database::log::{self, Log},
	global::GlobalState,
};

#[get("/logs/user/{username}/channel/{channel}")]
async fn user_logs(
	global_data: web::Data<GlobalState>,
	path: web::Path<(String, String)>,
	query: web::Query<ApiPaginationQuery>,
) -> Result<HttpResponse, Error> {
	let (username, channel) = path.into_inner();

	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_username(&global_data.db, &username, &channel, limit, offset)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(ApiPaginationResponse { offset, data: logs }))
}

#[derive(Deserialize)]
struct ChannelLogsQuery {
	message_id: String,
	offset: Option<i64>,
	limit: Option<i64>,
}

#[get("/logs/channel/{channel}")]
async fn channel_log(
	global_data: web::Data<GlobalState>,
	path: web::Path<String>,
	query: web::Query<ChannelLogsQuery>,
) -> Result<HttpResponse, Error> {
	let channel = path.into_inner();

	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_channel(&global_data.db, &query.message_id, &channel, limit, offset)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(ApiPaginationResponse { offset, data: logs }))
}

#[derive(Deserialize)]
struct SearchQuery {
	query: String,
}

#[get("/logs/search/users")]
async fn search_logs(
	global_data: web::Data<GlobalState>,
	query: web::Query<SearchQuery>,
) -> Result<HttpResponse, Error> {
	let users = log::search_users(&global_data.db, query.query.clone())
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(users))
}

#[get("/logs/user/{username}/active")]
async fn user_active_channels(
	global_data: web::Data<GlobalState>,
	path: web::Path<String>,
) -> Result<HttpResponse, Error> {
	let username = path.into_inner();

	let channels = log::get_active_channels(&global_data.db, &username)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(channels))
}

#[get("/logs/top/users")]
async fn top_users(global_data: web::Data<GlobalState>) -> Result<HttpResponse, Error> {
	let users = log::get_top_users(&global_data.db).await.unwrap();

	Ok(HttpResponse::Ok().json(users))
}

#[get("/logs/top/users/channel/{channel}")]
async fn top_users_channel(
	global_data: web::Data<GlobalState>,
	path: web::Path<String>,
) -> Result<HttpResponse, Error> {
	let channel = path.into_inner();

	let users = log::get_top_users_channel(&global_data.db, &channel)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(users))
}

#[get("/logs/top/channels")]
async fn top_channels(global_data: web::Data<GlobalState>) -> Result<HttpResponse, Error> {
	let channels = log::get_top_channels(&global_data.db).await.unwrap();

	Ok(HttpResponse::Ok().json(channels))
}
