use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
    #[cfg(not(target_os = "windows"))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strip: Option<bool>,
    /// When remote repo was last updated
    pub timestamp: DateTime<Utc>,
    /// asset name contais
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_glob: Option<String>,
    /// asset name matches RegEx
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_re: Option<String>,
    /// archive asset's entry name contains
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_glob: Option<String>,
    /// archive asset's entry name matches RegEx
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_re: Option<String>,
}

#[derive(Debug)]
pub enum PackageMatchKind {
    Exact,
    Latest,
    SemVer,
}

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
        serde_json::to_string_pretty(packages).context("serializing packages into JSON format")?,
    )
    .context(format!(
        "writing packages file: {}",
        packages_file.display()
    ))?;

    Ok(())
}
