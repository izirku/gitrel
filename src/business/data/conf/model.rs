use crate::foundation::consts;
use anyhow::{Context, Result};
use chrono::prelude::*;
use directories::BaseDirs;
use regex::Regex;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::{cmp::Ordering, collections::BTreeMap};

// use std::fs::{self, File};
// use clap::crate_name;
// use directories::ProjectDirs;

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub bin_dir: Option<String>,
    pub strip: Option<bool>,
}

pub fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile> {
    let base_dirs = BaseDirs::new().unwrap();
    let bin_dir = base_dirs.executable_dir().unwrap().to_string_lossy();

    match fs::read_to_string(&path) {
        Ok(config) => {
            toml::from_str(&config).with_context(|| format!("reading config: {:?}", path))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = ConfigFile {
                os: Some(consts::OS.to_string()),
                arch: Some(consts::ARCH.to_string()),
                bin_dir: Some(bin_dir.to_string()),
                strip: Some(true),
            };

            fs::write(&path, toml::to_string(&config)?)?;
            Ok(config)
        }
        Err(err) => Err(err).with_context(|| format!("unable to read config file: {:?}", path)),
    }
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
