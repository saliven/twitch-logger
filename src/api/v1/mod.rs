use actix_web::{get, http::Error, HttpResponse};

pub mod log;

#[get("/")]
async fn index() -> Result<HttpResponse, Error> {
	Ok(HttpResponse::Ok().body("API V1 Okayeg"))
}
