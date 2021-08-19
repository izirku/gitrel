use crate::foundation::consts;
use anyhow::{Context, Result};
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub bin_dir: Option<String>,
    pub strip: Option<bool>,
}

pub type PackageReqMap = BTreeMap<String, PackageReq>;
pub type PackageReqDetailMap = BTreeMap<String, PackageReqDetail>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PackageReq {
    Simple(String),
    Detailed(PackageReqDetail),
}

impl PackageReq {
    pub fn into_detailed(self, name: &str) -> PackageReqDetail {
        use PackageReq::*;
        match self {
            Simple(version) => PackageReqDetail {
                repo: Some(format!("{0}/{0}", name)),
                matches: version,
                ..Default::default()
            },
            Detailed(detailed) => PackageReqDetail {
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
pub struct PackageReqDetail {
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
    #[serde(rename = "version")]
    pub matches: String,
    pub date_fmt: Option<String>,
    #[serde(default = "strip_default")]
    pub strip: bool,
}

impl PackageReqDetail {
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
                    // requested = m.as_str().to_string();
                }
                None => return Ok(false),
            }
        } else {
            requested = self.matches.as_str();
        }
        dbg!(requested);

        // 3. try match
        match self.match_kind {
            PackageMatchKind::SemVer => {
                let ver_req = VersionReq::parse(requested)?;
                if let Some(m) = consts::SEMVER.find(candidate) {
                    let ver_remote = Version::parse(m.as_str())?;
                    if ver_req.matches(&ver_remote) {
                        return Ok(true);
                    }
                }
            }
            _ => unimplemented!(),
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
