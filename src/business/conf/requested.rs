use serde::Deserialize;
use std::collections::BTreeMap;

use super::PackageMatchKind;
use crate::business::{rx, util};

pub type PackageReqMap = BTreeMap<String, RequestedSpec>;
// pub type PackageReqDetailMap = BTreeMap<String, DetailedRequest>;

/// A Binary app request specification as it appears in `requested.toml`
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RequestedSpec {
    Simple(String),
    Detailed(DetailedRequest),
}

#[derive(Debug, Deserialize, Default)]
pub struct DetailedRequest {
    pub repo: Option<String>,
    pub filter: Option<String>,
    pub extract: Option<String>,
    #[serde(default)]
    #[serde(rename = "type")]
    pub match_kind: PackageMatchKind,
    /// This is interpreted differently, depending on the `match_kind`. Can be:
    ///
    /// - a *semver*: `^0.10.0`, etc
    /// - a *date*: `=2021-08-29` (op: `=`, `<`, `>`)
    /// - a *regex*
    /// - an exact *name*
    #[serde(default = "matches_default")]
    pub matches: String,
    pub date_fmt: Option<String>,
    #[serde(default = "strip_default")]
    pub strip: bool,
}

fn matches_default() -> String {
    "*".to_string()
}

fn strip_default() -> bool {
    true
}

impl RequestedSpec {
    pub fn from_str(str: &str) -> Self {
        let match_kind;
        let (repo, tag) = util::parse_gh_repo_spec(str);

        if rx::SEMVER.is_match(&tag) {
            match_kind = PackageMatchKind::SemVer;
        } else {
            match_kind = PackageMatchKind::Named;
        }
        Self::Detailed(DetailedRequest {
            repo: Some(repo),
            match_kind,
            matches: tag,
            // TODO: get it from the configuration manager
            strip: true,
            ..Default::default()
        })
    }

    // pub fn into_detailed(self, name: &str) -> DetailedRequest {
    //     use RequestedSpec::*;
    //     match self {
    //         Simple(version) => DetailedRequest {
    //             repo: Some(format!("{0}/{0}", name)),
    //             matches: version,
    //             ..Default::default()
    //         },
    //         Detailed(detailed) => DetailedRequest {
    //             repo: detailed.repo.or(Some(format!("{0}/{0}", name))),
    //             ..detailed
    //         },
    //     }
    // }

    //     pub fn get_repo(&self, name: &str) -> String {
    //         use PackageReq::*;
    //         match self {
    //             Simple(_) => format!("{0}/{0}", name),
    //             Detailed(detailed) => match detailed.repo {
    //                 None => format!("{0}/{0}", name),
    //                 Some(ref repo) => repo.to_string(),
    //             },
    //         }
    //     }
    //
    //     pub fn get_version(&self) -> String {
    //         use PackageReq::*;
    //         match self {
    //             Simple(version) => version.to_owned(),
    //             // Detailed(detailed) => detailed.version_requested.as_ref().unwrap_or("*").clone(),
    //             Detailed(detailed) => match detailed.version_requested {
    //                 None => "*".to_string(),
    //                 Some(ref ver) => ver.to_string(),
    //             },
    //         }
    //     }
}
