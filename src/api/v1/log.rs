use actix_web::{get, http::Error, web, HttpResponse};

use crate::{database::log::Log, global::GlobalState};

#[get("/logs/{username}/{channel}")]
async fn user_logs(
	data: web::Data<GlobalState>,
	path: web::Path<(String, String)>,
) -> Result<HttpResponse, Error> {
	let (username, channel) = path.into_inner();

	let logs = Log::get_by_username(&data.db, &username, &channel, 100, 0)
		.await
		.unwrap();

	Ok(HttpResponse::Ok().json(logs))
}
