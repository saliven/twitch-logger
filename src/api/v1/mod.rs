use actix_web::{get, http::Error, HttpResponse};

pub mod chart;
pub mod log;
pub mod stats;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
	Ok(HttpResponse::Ok().body("Okayeg"))
}
