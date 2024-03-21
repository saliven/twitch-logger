use std::io::Error;

use anyhow::Result;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use tracing::info;

use crate::{api::v1::index, global::GlobalState};

mod v1;

#[derive(Deserialize, Serialize)]
pub struct ApiPaginationQuery {
	pub offset: Option<i64>,
	pub limit: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct ApiPaginationResponse<T> {
	pub offset: i64,
	pub data: T,
}

pub async fn start(global: GlobalState) -> Result<(), Error> {
	let port = std::env::var("PORT")
		.map(|s| s.parse().unwrap_or(4001))
		.unwrap_or(4001);

	info!("Starting API server on port {}", port);

	let log_routes = Router::new()
		.route("/user/:username/channel/:channel", get(v1::log::user_logs))
		.route("/channel/:channel", get(v1::log::channel_log))
		.route("/search/users", get(v1::log::search_logs))
		.route("/user/:username/active", get(v1::log::user_active_channels))
		.route("/top/users", get(v1::log::top_users))
		.route("/top/channels", get(v1::log::top_channels))
		.route(
			"/top/users/channel/:channel",
			get(v1::log::top_users_channel),
		)
		.route("/history/:username", get(v1::log::username_history));

	let v1_routes = Router::new()
		.nest("/logs", log_routes)
		.route("/stats/size", get(v1::stats::stats_size))
		.route("/chart/rate", get(v1::chart::rate_chart));

	let app = Router::new()
		.nest("/api/v1", v1_routes)
		.route("/", get(index))
		.layer(ServiceBuilder::new().layer(CorsLayer::new().allow_origin(Any)))
		.with_state(global);

	let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
		.await
		.unwrap();

	axum::serve(listener, app).await
}
