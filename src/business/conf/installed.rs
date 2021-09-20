use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type InstalledPackageMap = BTreeMap<String, InstalledPackage>;

#[derive(Debug, Deserialize, Serialize)]
pub struct InstalledPackage {
    pub repo: String,
    pub tag: String,
    pub modified_at: DateTime<Utc>,
}

impl InstalledPackage {
    pub fn create(repo: &str, tag: &str, modified_at: &DateTime<Utc>) -> Self {
        Self {
            repo: repo.to_owned(),
            tag: tag.to_owned(),
            modified_at: modified_at.to_owned(),
        }
    }
}
