use super::asset::Asset;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Release {
    // pub url: Url,
    // pub html_url: Url,
    pub assets_url: Url,
    pub upload_url: Url,
    // pub tarball_url: Option<Url>,
    // pub zipball_url: Option<Url>,
    pub id: u64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    // pub name: Option<String>,
    // pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub published_at: DateTime<Utc>,
    // pub published_at: Option<DateTime<Utc>>,
    // pub author: crate::models::User,
    pub assets: Vec<Asset>,
}

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "snake_case")]
// #[non_exhaustive]
// pub struct Tag {
//     pub name: String,
//     // pub commit: CommitObject,
//     // pub zipball_url: Url,
//     // pub tarball_url: Url,
//     pub node_id: String,
// }

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "snake_case")]
// #[non_exhaustive]
// pub struct CommitObject {
//     pub sha: String,
//     pub url: Url,
// }
