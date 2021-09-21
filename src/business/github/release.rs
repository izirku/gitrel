use super::asset::Asset;
use crate::business::conf::Package;
use crate::business::rx;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

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

impl Release {
    /// Given a `package` spec, see if we have a SemVer match.
    pub fn matches_semver(&self, package: &Package) -> Result<bool> {
        if let Some(extacted_remote_semver) = rx::SEMVER.find(&self.tag_name) {
            let ver_remote = semver::Version::parse(extacted_remote_semver.as_str())
                .context("parsing tag as semver")?;
            let ver_req = semver::VersionReq::parse(&package.requested).unwrap();
            return Ok(ver_req.matches(&ver_remote));
        }
        Ok(false)
    }
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
