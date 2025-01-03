use core::str;
use std::collections::HashMap;

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use poem_openapi::{
	payload::Json,
	types::{ParseFromJSON, ToJSON},
	Object,
};
use serde::Serialize;

#[derive(Debug, Serialize, Object)]
pub struct PaginatedResponse<T: ParseFromJSON + ToJSON> {
	pub data: Vec<T>,
	pub cursors: Option<Cursors>,
}

#[derive(Debug, Serialize, Object)]
pub struct Cursors {
	pub after: Option<String>,
	pub before: Option<String>,
}

pub trait Pageable {
	fn get_cursor_values(&self) -> HashMap<String, String>;
}

pub fn encode_cursor(values: &HashMap<String, String>) -> String {
	let values: Vec<String> = values.iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
	URL_SAFE.encode(values.join(","))
}

pub fn decode_cursor(cursor: &str) -> HashMap<String, String> {
	let bytes = URL_SAFE.decode(cursor).unwrap();

	let cursor = str::from_utf8(&bytes).unwrap();

	cursor
		.split(',')
		.filter_map(|pair| {
			let (key, value) = pair.split_once(':')?;
			Some((key.to_string(), value.to_string()))
		})
		.collect()
}

pub async fn paginate<T, F, Fut>(
	cursor: Option<String>,
	limit: Option<u32>,
	fetch_data: F,
) -> Result<Json<PaginatedResponse<T>>>
where
	T: Serialize + Pageable + ParseFromJSON + ToJSON,
	F: Fn(Option<String>, u32) -> Fut,
	Fut: std::future::Future<Output = Result<Vec<T>, anyhow::Error>>,
{
	let limit = limit.unwrap_or(100);

	let items = fetch_data(cursor.clone(), limit).await?;

	// TODO: cursor resolution should be smarter and count the number of items
	let before_cursor = if items.len() == limit as usize {
		let last_item = items.last().unwrap();
		let cursor_values = last_item.get_cursor_values();
		let cursor = encode_cursor(&cursor_values);
		Some(cursor)
	} else {
		None
	};

	let response = PaginatedResponse {
		data: items,
		cursors: Some(Cursors {
			after: None,
			before: before_cursor,
		}),
	};

	Ok(Json(response))
}
