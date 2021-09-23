use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub type PackageMap = BTreeMap<String, Package>;

/// Is a representation of a \[maybe installed\] package. Also serves as
/// an interchange format between [ConfigurationManager](crate::business::conf::ConfigurationManager),
/// [GitHub](crate::business::github::GitHub),
/// and [Installer](crate::business::installer::Installer).
#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    /// lower cased *repository name*
    #[serde(skip)]
    pub name: Option<String>,
    /// is `repo_user/repo_name`
    pub repo: String,
    /// *release tag* of an installed or a *matched* release 
    pub tag: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    /// a requested *version*, can be one of:
    /// - `"*"` - latest release (default)
    /// - `"<plain string>"` - a named release (can be a pre-release)
    /// - `"<SEMVER string>"` - a *semver* to match against *release tag*
    pub requested: String,
}

#[derive(Debug)]
pub enum PackageMatchKind {
    Exact,
    Latest,
    SemVer,
}

impl Package {
    pub fn create(repo_spec: &str) -> Self {
        let (repo, repo_name, requested) = parse_gh_repo_spec(repo_spec);

        Self {
            name: Some(repo_name),
            repo,
            requested,
            tag: None,
            published_at: None,
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

/// Returns a triple (repo, repo_name, requested)
fn parse_gh_repo_spec(repo_spec: &str) -> (String, String, String) {
    let (repo, tag) = if repo_spec.contains('@') {
        let (repo, tag) = repo_spec.split_at(repo_spec.find('@').unwrap());
        (repo, tag.trim_start_matches('@'))
    } else {
        (repo_spec, "*")
    };

    let (repo, name) = if repo.contains('/') {
        (
            repo.to_owned(),
            repo.split_at(repo.find('/').unwrap())
                .1
                .get(1..)
                .unwrap()
                .to_lowercase(),
        )
    } else {
        (format!("{0}/{0}", repo), repo.to_lowercase())
    };

    (repo, name, tag.to_owned())
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
