use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

use super::asset::Asset;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Release {
    pub assets_url: Url,
    pub upload_url: Url,
    pub id: u64,
    pub node_id: String,
    pub tag_name: String,
    pub target_commitish: String,
    pub draft: bool,
    pub prerelease: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub published_at: DateTime<Utc>,
    pub assets: Vec<Asset>,
}
