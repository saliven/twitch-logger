use actix_web::{get, web, Error, HttpResponse};

use crate::{database::chart, global::GlobalState};

#[get("/chart/rate")]
async fn rate_chart(global_data: web::Data<GlobalState>) -> Result<HttpResponse, Error> {
	let data = chart::get_rate_chart(&global_data.db).await.unwrap();

	Ok(HttpResponse::Ok().json(data))
}
