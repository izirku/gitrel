use crate::business::data::conf::model::{PackageReqDetail, PackageReqMap};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

// use toml::Value;

/// List requested packages in a given TOML `file` file.
pub fn list_requested(file: &Path) -> Result<()> {
    let file = fs::read_to_string(file)
        .with_context(|| format!("unable to read packages file: {:?}", file))?;

    // let toml = toml::from_str::<Value>(&file).with_context(|| format!("malformed packages TOML file: {:?}", file))?;
    let toml: Vec<PackageReqDetail> = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", file))?
        .into_iter()
        .map(|(name, pkg_spec)| pkg_spec.into_detailed(name))
        .collect();
    dbg!(toml);

    Ok(())
}
