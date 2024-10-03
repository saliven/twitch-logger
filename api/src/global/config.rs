use figment::{
	providers::{Env, Format, Toml},
	Figment,
};
use serde::Deserialize;

use crate::ENV;

#[derive(Debug, Deserialize, Clone)]
pub struct ClickhouseConfig {
	pub url: String,
	pub database: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
	pub clickhouse: ClickhouseConfig,
}

pub fn load() -> Config {
	let mut config = Figment::new();

	if ENV.as_str() == "development" {
		config = config.merge(Toml::file("./api/development.toml"));
	} else {
		config = config.merge(Env::raw().split("_"));
	}

	config.extract().unwrap()
}
