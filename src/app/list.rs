use crate::business::data::conf::model::{PackageReqDetail, PackageReqDetailMap, PackageReqMap};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

// use toml::Value;

/// List requested packages in a given TOML `file` file.
pub fn list_requested(file: &Path) -> Result<()> {
    let file = fs::read_to_string(file)
        .with_context(|| format!("unable to read packages file: {:?}", file))?;

    // let toml = toml::from_str::<Value>(&file).with_context(|| format!("malformed packages TOML file: {:?}", file))?;
    // let toml: Vec<PackageReqDetail> = toml::from_str::<PackageReqMap>(&file)
    //     .with_context(|| format!("malformed packages TOML file: {:?}", file))?
    //     .into_iter()
    //     .map(|(name, pkg_spec)| pkg_spec.into_detailed(name))
    //     .collect();
    let toml = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", file))?;

    let max_width_name: usize = toml
        .keys()
        .max_by_key(|bin_name| bin_name.len())
        .and_then(|bin_name| Some(bin_name.len()))
        .unwrap();
    let max_width_ver: usize = toml
        .values()
        .max_by_key(|spec| spec.get_version().len())
        .and_then(|spec| Some(spec.get_version().len()))
        .unwrap();
    // let calc_pad = |bin_name: &str| max_width - bin_name.len();

    for (name, pkg_spec) in toml.into_iter() {
        let ver = format!("@\"{}\"", pkg_spec.get_version());
        let details = pkg_spec.into_detailed(&name);
        println!(
            "{:<w_name$} {:>w_ver$} [https://github.com/{}]",
            &name,
            &ver,
            &details.repo.unwrap_or("Err".to_string()),
            w_name = max_width_name,
            w_ver = max_width_ver,
            // width = calc_pad(&name),
        );
    }
    // dbg!(toml);

    Ok(())
}
