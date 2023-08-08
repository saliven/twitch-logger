use actix_web::{get, web, Error, HttpResponse};

use crate::{database::stats::get_size, global::GlobalState};

#[get("/stats/size")]
async fn stats_size(global_data: web::Data<GlobalState>) -> Result<HttpResponse, Error> {
	let stats = get_size(&global_data.db).await.unwrap();

	Ok(HttpResponse::Ok().json(stats))
}
