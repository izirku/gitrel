use anyhow::{Context, Result};
use reqwest::{header, Client};

pub fn create(token: &Option<String>) -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github.v3+json"),
    );
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("reqwest"),
    );
    if let Some(token) = token {
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(token).unwrap(),
        );
    }

    reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .with_context(|| "creating REST API client has failed.")
}
