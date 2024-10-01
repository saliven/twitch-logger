use poem::http;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};

#[derive(Debug, Clone, Hash, Eq, PartialEq, EncodeLabelSet)]
pub struct HttpRequest {
	pub path: String,
	pub method: HttpMethod,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, EncodeLabelValue)]
pub enum HttpMethod {
	GET,
	POST,
	PUT,
	DELETE,
}

impl From<http::Method> for HttpMethod {
	fn from(method: http::Method) -> Self {
		match method {
			http::Method::GET => HttpMethod::GET,
			http::Method::POST => HttpMethod::POST,
			http::Method::PUT => HttpMethod::PUT,
			http::Method::DELETE => HttpMethod::DELETE,
			_ => HttpMethod::GET,
		}
	}
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, EncodeLabelSet)]
pub struct HttpResponse {
	pub status_code: u16,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, EncodeLabelSet)]
pub struct DatabaseQuery {
	pub query: String,
}
