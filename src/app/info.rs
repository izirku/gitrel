use crate::business::data::github;
use reqwest::header;

pub async fn info(repo: &str, token: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
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
            header::HeaderValue::from_str(token.as_str()).unwrap(),
        );
    }

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let releases = client
        .get(format!("https://api.github.com/repos/{}/releases", repo))
        .send()
        .await?
        .json::<Vec<github::model::Release>>()
        .await?;

    println!("{:#?}", &releases[..2]);
    println!("{}", releases.len());

    Ok(())
}
