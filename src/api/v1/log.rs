use actix_web::{get, http::Error, web, HttpResponse};

use crate::{api::ApiPagination, database::log::Log, global::GlobalState};

#[get("/logs/{username}/{channel}")]
async fn user_logs(
	data: web::Data<GlobalState>,
	path: web::Path<(String, String)>,
	query: web::Query<ApiPagination>,
) -> Result<HttpResponse, Error> {
	let (username, channel) = path.into_inner();

	let offset = query.offset.unwrap_or(0);
	let limit = query.limit.unwrap_or(100);

	let logs = Log::get_by_username(&data.db, &username, &channel, limit, offset)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(logs))
}
