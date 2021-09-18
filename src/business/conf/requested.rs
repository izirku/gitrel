use serde::Deserialize;
use std::collections::BTreeMap;

use crate::business::util::parse_gh_repo_spec;

pub type PackageReqMap = BTreeMap<String, RequestedPackage>;
// pub type PackageReqDetailMap = BTreeMap<String, DetailedRequest>;

/// A Binary app request specification as it appears in `requested.toml`
#[derive(Debug, Deserialize, Default)]
pub struct RequestedPackage {
    pub repo: String,
    /// This is interpreted differently, i.e.:
    ///
    /// - latest release: `"*"` (default)
    /// - a *semver*: `"^0.10.0"`, etc
    /// - an exact *name*: `"nightly"` (includes pre-releases)
    #[serde(default = "matches_default")]
    pub version: String,
    #[serde(default)]
    pub strip: bool,
}

fn matches_default() -> String {
    "*".to_string()
}

impl RequestedPackage {
    pub fn create(repo: &str, strip: bool) -> Self {
        let (repo, tag) = parse_gh_repo_spec(repo);
        Self {
            repo,
            version: tag,
            strip,
        }
    }
}
