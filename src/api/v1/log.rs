use actix_web::{get, http::Error, web, HttpResponse};

use crate::{api::ApiPagination, database::log::Log, global::GlobalState};

#[get("/logs/{username}/{channel}")]
async fn user_logs(
	global_data: web::Data<GlobalState>,
	path: web::Path<(String, String)>,
	query: web::Query<ApiPagination>,
) -> Result<HttpResponse, Error> {
	let (username, channel) = path.into_inner();

	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_username(&global_data.db, &username, &channel, limit, offset)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(logs))
}

#[get("/channels/{username}")]
async fn user_active_channels(
	global_data: web::Data<GlobalState>,
	path: web::Path<String>,
) -> Result<HttpResponse, Error> {
	let username = path.into_inner();

	let channels = Log::get_active_channels(&global_data.db, &username)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(channels))
}

#[get("/top")]
async fn top_users(global_data: web::Data<GlobalState>) -> Result<HttpResponse, Error> {
	let top_users = Log::get_top_users(&global_data.db).await.unwrap();

	Ok(HttpResponse::Ok().json(top_users))
}
