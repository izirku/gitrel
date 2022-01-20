use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Representation an installed package.
#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    /// user name
    pub user: String,
    /// repo name
    pub repo: String,
    /// name binary to use
    pub bin_name: String,
    /// *release tag* of an installed or a *matched* release
    pub tag: String,
    /// a requested *version*, can be one of:
    /// - `"*"` - latest release (default)
    /// - `"<plain string>"` - a named release (can be a pre-release)
    /// - `"<SEMVER string>"` - a *semver* to match against *release tag*
    pub requested: String,
    /// use `strip` on the binary
    pub strip: Option<bool>,
    /// When remote repo was last updated
    pub timestamp: DateTime<Utc>,
    /// asset name contais
    pub asset_contains: Option<String>,
    /// asset name matches RegEx
    pub asset_re: Option<String>,
    /// archive asset's entry name contains
    pub entry_contains: Option<String>,
    /// archive asset's entry name matches RegEx
    pub entry_re: Option<String>,

    // /// Used by GitHub APIs to identify and download an asset
    // #[serde(skip)]
    // pub asset_id: Option<String>,
    // /// Used to name a downloaded archive, and to determine how to extract it
    // #[serde(skip)]
    // pub asset_name: Option<String>,
    // #[serde(skip)]
    // pub asset_path: Option<PathBuf>,
}

#[derive(Debug)]
pub enum PackageMatchKind {
    Exact,
    Latest,
    SemVer,
}

// impl Package {
//     // pub fn create(repo_spec: &str, strip: Option<bool>) -> Result<Self> {
//     //     let (repo, repo_name, requested) = parse_gh_repo_spec(repo_spec)?;

//     //     Ok(Self {
//     //         name: Some(repo_name),
//     //         repo,
//     //         tag: None,
//     //         requested,
//     //         strip,
//     //         timestamp: None,
//     //         asset_id: None,
//     //         asset_name: None,
//     //         asset_path: None,
//     //     })
//     // }

// }

pub fn match_kind(requested: &str) -> PackageMatchKind {
    if requested == "*" {
        PackageMatchKind::Latest
    } else if semver::VersionReq::parse(requested).is_ok() {
        PackageMatchKind::SemVer
    } else {
        PackageMatchKind::Exact
    }
}

pub fn read_packages_file(packages_file: &Path) -> Result<Vec<Package>> {
    match fs::read_to_string(packages_file) {
        Ok(s) => Ok(serde_json::from_str(&s).context(format!(
            "malformed packages JSON file: {}",
            packages_file.display()
        ))?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(_e) => Err(anyhow!(format!(
            "unable to read packages file: {}",
            packages_file.display()
        ))),
    }
}

pub fn write_packages_file(packages_file: &Path, packages: &[Package]) -> Result<()> {
    fs::write(
        packages_file,
        serde_json::to_string(packages).context("serializing packages into JSON format")?,
    )
    .context(format!(
        "writing packages file: {}",
        packages_file.display()
    ))?;

    Ok(())
}
