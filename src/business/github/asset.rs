use crate::business::rx;
use crate::Result;
use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Asset {
    // pub url: Url,
    pub browser_download_url: Url,
    pub id: u64,
    pub node_id: String,
    pub name: String,
    pub label: Option<String>,
    pub state: String,
    pub content_type: String,
    pub size: i64,
    pub download_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // pub uploader: User,
}

impl Asset {
    pub fn is_match(&self) -> bool {
        rx::MATCH_OS.is_match(&self.name)
            && rx::MATCH_ARCH.is_match(&self.name)
            && rx::MATCH_ABI.is_match(&self.name)
    }
}
