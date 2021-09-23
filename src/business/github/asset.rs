use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;
// use url::Url;
// use crate::business::rx;
// use crate::Result;
// use anyhow::Context;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

lazy_static! {
    static ref TERMS: Regex =
        Regex::new(r"(x86_64|x86\-64|[a-zA-Z0-9]+)").expect("error parsing regex");
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Asset {
    // pub url: Url,
    // pub browser_download_url: Url,
    pub id: u64,
    // pub node_id: String,
    pub name: String,
    // pub label: Option<String>,
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
        for term in TERMS.find_iter(&self.name.to_lowercase()) {
            if EXCLUDE_SET.contains(term.as_str()) {
                return false;
            }
        }
        true
        //     rx::MATCH_OS.is_match(&self.name)
        //         && rx::MATCH_ARCH.is_match(&self.name)
        //         && rx::MATCH_ABI.is_match(&self.name)
    }
}
