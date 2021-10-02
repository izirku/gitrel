use super::util::parse_gh_repo_spec;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use url::Url;

pub type PackageMap = BTreeMap<String, Package>;

/// Is a representation of a \[maybe installed\] package. Also serves as
/// an interchange format between [ConfigurationManager](crate::business::conf::ConfigurationManager),
/// [GitHub](crate::business::github::GitHub),
/// and [Installer](crate::business::installer::Installer).
#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    // lower cased *repository name*
    // #[serde(skip)]
    // pub name: Option<String>,
    /// is `repo_user/repo_name`
    pub repo: Url,
    /// *release tag* of an installed or a *matched* release
    pub tag: Option<String>,
    /// a requested *version*, can be one of:
    /// - `"*"` - latest release (default)
    /// - `"<plain string>"` - a named release (can be a pre-release)
    /// - `"<SEMVER string>"` - a *semver* to match against *release tag*
    pub requested: String,
    /// When remote repo was last updated
    pub timestamp: Option<DateTime<Utc>>,
    /// Used by GitHub APIs to identify and download an asset
    #[serde(skip)]
    pub asset_id: Option<String>,
    /// Used to name a downloaded archive, and to determine how to extract it
    #[serde(skip)]
    pub asset_name: Option<String>,
    #[serde(skip)]
    pub asset_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum PackageMatchKind {
    Exact,
    Latest,
    SemVer,
}

impl Package {
    pub fn create(repo_spec: &str) -> Self {
        // let (repo, repo_name, requested) = parse_gh_repo_spec(repo_spec);
        let (repo, requested) = parse_gh_repo_spec(repo_spec);

        Self {
            // name: Some(repo_name),
            repo,
            tag: None,
            requested,
            timestamp: None,
            asset_id: None,
            asset_name: None,
            asset_path: None,
        }
    }

    pub fn match_kind(&self) -> PackageMatchKind {
        if self.requested == "*" {
            PackageMatchKind::Latest
        } else if semver::VersionReq::parse(&self.requested).is_ok() {
            PackageMatchKind::SemVer
        } else {
            PackageMatchKind::Exact
        }
    }
}

// fn parse_gh_repo_name(str: &str) -> String {
//     // TODO: add regex validation here, wrap in Result<_>?
//     if str.contains('/') {
//         str.to_owned()
//     } else {
//         format!("{0}/{0}", str)
//     }
// }

// fn parse_gh_repo_spec(str: &str) -> (String, String) {
//     // TODO: add regex validation here, wrap in Result<_>?
//     if str.contains('@') {
//         let (name, tag) = str.split_at(str.find('@').unwrap());
//         (
//             parse_gh_repo_name(name),
//             tag.trim_start_matches('@').to_owned(),
//         )
//     } else {
//         (parse_gh_repo_name(str), "*".to_owned())
//     }
// }
