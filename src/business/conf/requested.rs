use anyhow::{Context, Result};
use chrono::prelude::*;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::Deserialize;
use std::{cmp::Ordering, collections::BTreeMap};

use crate::business::github::model::Release;
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackageMatchKind {
    /// Release tag follows a well behaved semver
    SemVer,
    /// compare a date of a release tag update and what we have installed
    Date,
    /// including pre-releases (i.e. "nightly" tag commonly set to be a pre-release)
    Named,
    /// Latest non pre-release
    Latest,
    /// Release tag matches a certain regex
    RegEx,
}

fn matches_default() -> String {
    "*".to_string()
}

fn strip_default() -> bool {
    true
}

impl Default for PackageMatchKind {
    fn default() -> Self {
        Self::SemVer
    }
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

    /// Given a *candidate* GitHub `Release` , see if it to matches our requestesd spec.
    pub fn matches(&self, candidate: &Release) -> Result<bool> {
        match self {
            // in a simple case, we do a semver match, falling back to an
            // exact match attempt in order to handle something like:
            //   `rust-analyzer = nightly`
            Self::Simple(tag) => {
                if let Ok(ver_req) = VersionReq::parse(tag) {
                    if let Some(m) = rx::SEMVER.find(&candidate.tag_name) {
                        let ver_remote = Version::parse(m.as_str())?;
                        return Ok(ver_req.matches(&ver_remote));
                    }
                } else {
                    if tag == &candidate.tag_name {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            Self::Detailed(details) => {
                let requested;
                // 1. maybe filter
                if let Some(expr) = &details.filter {
                    let re = Regex::new(expr).with_context(|| {
                        format!("bad filter RegEx: {}", details.filter.as_ref().unwrap())
                    })?;
                    if !re.is_match(&candidate.tag_name) {
                        return Ok(false);
                    }
                }
                // 2. maybe extract
                if let Some(expr) = &details.extract {
                    let re = Regex::new(expr).with_context(|| {
                        format!("bad filter RegEx: {}", details.filter.as_ref().unwrap())
                    })?;
                    match re.find(&candidate.tag_name) {
                        Some(m) => {
                            requested = m.as_str();
                        }
                        None => return Ok(false),
                    }
                } else {
                    requested = details.matches.as_str();
                }
                // dbg!(requested);

                // 3. try match
                match details.match_kind {
                    PackageMatchKind::Named => {
                        if requested == &candidate.tag_name {
                            return Ok(true);
                        }
                    }
                    PackageMatchKind::RegEx => {
                        let re = Regex::new(requested)?;
                        if re.is_match(&candidate.tag_name) {
                            return Ok(true);
                        }
                    }
                    PackageMatchKind::SemVer => {
                        let ver_req = VersionReq::parse(requested)?;
                        if let Some(m) = rx::SEMVER.find(&candidate.tag_name) {
                            let ver_remote = Version::parse(m.as_str())?;
                            if ver_req.matches(&ver_remote) {
                                return Ok(true);
                            }
                        }
                    }
                    PackageMatchKind::Date => {
                        // TODO: better OP & date ranges parsing, allow `>=`, `>=DT_START & <=DT_END`
                        let (op, dt_str) = requested.split_at(1);
                        let dt_req =
                            NaiveDate::parse_from_str(dt_str, details.date_fmt.as_ref().unwrap())?;
                        if let Ok(dt_remote) = NaiveDate::parse_from_str(
                            &candidate.tag_name,
                            details.date_fmt.as_ref().unwrap(),
                        ) {
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
