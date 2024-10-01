use poem::Route;
use poem_extensions::UniOpenApi;
use poem_openapi::OpenApiService;

// pub mod chart;
pub mod logs;
// pub mod stats;

#[derive(UniOpenApi)]
pub struct ApisUnion(logs::Logs);

pub fn route() -> Route {
	let apis = ApisUnion(logs::Logs);

	let service = OpenApiService::new(apis, "Logger v1 API", "v1.0").server("/v1");

	let ui = service.rapidoc();
	let spec = service.spec_endpoint();

	Route::new()
		.at("/spec", spec)
		.at("/ui", ui)
		.nest("/", service)
}
