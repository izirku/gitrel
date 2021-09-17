use anyhow::{Context, Result};
use chrono::prelude::*;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::BTreeMap};

use crate::foundation::consts;

pub type PackageReqMap = BTreeMap<String, RequestedSpec>;
pub type PackageReqDetailMap = BTreeMap<String, DetailedRequest>;

/// A Binary app request specification as it appears in `requested.toml`
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RequestedSpec {
    Simple(String),
    Detailed(DetailedRequest),
}

// impl From<Requested> for DetailedRequest {
//     fn from(requested: Requested) -> Self {

//     }
// }

impl RequestedSpec {
    pub fn into_detailed(self, name: &str) -> DetailedRequest {
        use RequestedSpec::*;
        match self {
            Simple(version) => DetailedRequest {
                repo: Some(format!("{0}/{0}", name)),
                matches: version,
                ..Default::default()
            },
            Detailed(detailed) => DetailedRequest {
                repo: detailed.repo.or(Some(format!("{0}/{0}", name))),
                ..detailed
            },
        }
    }

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

impl DetailedRequest {
    pub fn matches(&self, candidate: &str) -> Result<bool> {
        let requested;
        // 1. maybe filter
        if let Some(expr) = &self.filter {
            let re = Regex::new(&expr)
                .with_context(|| format!("bad filter RegEx: {}", self.filter.as_ref().unwrap()))?;
            if !re.is_match(candidate) {
                return Ok(false);
            }
        }
        // 2. maybe extract
        if let Some(expr) = &self.extract {
            let re = Regex::new(&expr)
                .with_context(|| format!("bad filter RegEx: {}", self.filter.as_ref().unwrap()))?;
            match re.find(candidate) {
                Some(m) => {
                    requested = m.as_str();
                }
                None => return Ok(false),
            }
        } else {
            requested = self.matches.as_str();
        }
        // dbg!(requested);

        // 3. try match
        match self.match_kind {
            PackageMatchKind::Named => {
                if requested == candidate {
                    return Ok(true);
                }
            }
            PackageMatchKind::RegEx => {
                let re = Regex::new(requested)?;
                if re.is_match(candidate) {
                    return Ok(true);
                }
            }
            PackageMatchKind::SemVer => {
                let ver_req = VersionReq::parse(requested)?;
                if let Some(m) = consts::SEMVER.find(candidate) {
                    let ver_remote = Version::parse(m.as_str())?;
                    if ver_req.matches(&ver_remote) {
                        return Ok(true);
                    }
                }
            }
            PackageMatchKind::Date => {
                let (op, dt_str) = requested.split_at(1);
                let dt_req = NaiveDate::parse_from_str(dt_str, self.date_fmt.as_ref().unwrap())?;
                if let Ok(dt_remote) =
                    NaiveDate::parse_from_str(candidate, self.date_fmt.as_ref().unwrap())
                {
                    match (op, dt_remote.cmp(&dt_req)) {
                        ("=", Ordering::Equal) => return Ok(true),
                        ("<", Ordering::Less) => return Ok(true),
                        (">", Ordering::Greater) => return Ok(true),
                        _ => return Ok(false),
                    }
                }
            }
        }
        Ok(false)
    }
}

fn matches_default() -> String {
    "*".to_string()
}

fn strip_default() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageMatchKind {
    SemVer,
    Date,
    Named,
    RegEx,
}

impl Default for PackageMatchKind {
    fn default() -> Self {
        Self::SemVer
    }
}

// #[serde(from = "PackageSpec")]
// pub struct Package {}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InstalledPackage {}
