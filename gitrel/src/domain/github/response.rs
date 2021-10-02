use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum GithubResponse<T> {
    Ok(T),
    Err(ErrorResponse),
}

// #[derive(Serialize, Deserialize)]
#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
    message: String,
    // documentation_url: String,
}
