use serde::Deserialize;
use serde_json::Value;
use std::fmt;

use crate::auth::Credentials;

#[derive(Debug)]
pub enum Error {
	Http(reqwest::Error),
	QueryFailed(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Http(e) => write!(f, "{}", e),
			Self::QueryFailed(msg) => write!(f, "{}", msg),
		}
	}
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
	fn from(e: reqwest::Error) -> Self {
		Self::Http(e)
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
	pub total_size: u64,
	pub done: bool,
	pub records: Vec<Value>,
	pub next_records_url: Option<String>,
}

pub async fn execute_query(creds: &Credentials, soql: &str) -> Result<QueryResult, Error> {
	let client = reqwest::Client::new();
	let url = format!("{}/services/data/v62.0/query", creds.instance_url);

	let resp = client
		.get(&url)
		.bearer_auth(&creds.access_token)
		.query(&[("q", soql)])
		.send()
		.await?;

	if !resp.status().is_success() {
		let status = resp.status();
		let body = resp.text().await.unwrap_or_default();
		return Err(Error::QueryFailed(format!("Query failed ({}): {}", status, body)));
	}

	let result: QueryResult = resp.json().await?;
	Ok(result)
}

pub async fn fetch_all(creds: &Credentials, soql: &str) -> Result<QueryResult, Error> {
	let mut result = execute_query(creds, soql).await?;

	while let Some(next_url) = result.next_records_url.take() {
		let client = reqwest::Client::new();
		let url = format!("{}{}", creds.instance_url, next_url);

		let resp = client.get(&url).bearer_auth(&creds.access_token).send().await?;

		if !resp.status().is_success() {
			let status = resp.status();
			let body = resp.text().await.unwrap_or_default();
			return Err(Error::QueryFailed(format!("Query pagination failed ({}): {}", status, body)));
		}

		let next: QueryResult = resp.json().await?;
		result.records.extend(next.records);
		result.next_records_url = next.next_records_url;
		result.done = next.done;
	}

	Ok(result)
}
