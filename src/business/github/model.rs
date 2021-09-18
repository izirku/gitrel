use crate::business::conf::Package;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReleaseResponse {
  Ok(Release),
  Err(ErrorResponse),
}

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    message: String,
    documentation_url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
    pub published_at: Option<DateTime<Utc>>,
    // pub author: crate::models::User,
    pub assets: Vec<Asset>,
}

impl Release {
    /// Given a GitHub `release` and a `package` spec, see if we have a match.
    pub fn matches(&self, package: &Package) -> Result<bool> {
        if let Some(req) = package.requested {
            if let Ok(ver_req) = semver::VersionReq::parse(&req.version) {
                let ver_remote = semver::Version::parse(&self.tag_name)?;
                return Ok(ver_req.matches(&ver_remote));
            } else {
                if &req.version == &self.tag_name {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

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

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Tag {
    pub name: String,
    // pub commit: CommitObject,
    // pub zipball_url: Url,
    // pub tarball_url: Url,
    pub node_id: String,
}

// #[derive(Debug, Clone, PartialEq, Deserialize)]
// #[serde(rename_all = "snake_case")]
// #[non_exhaustive]
// pub struct CommitObject {
//     pub sha: String,
//     pub url: Url,
// }
