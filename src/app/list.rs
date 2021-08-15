use crate::business::data::conf::model::PackageReqMap;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// List requested packages in a given TOML `file` file.
pub fn list_requested(file: &Path) -> Result<()> {
    let file = fs::read_to_string(file)
        .with_context(|| format!("unable to read packages file: {:?}", file))?;

    let toml = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", file))?;

    let mut cols = Vec::with_capacity(toml.len());

    for (name, pkg_spec) in toml.into_iter() {
        let ver = format!("@ {}", pkg_spec.get_version());
        let repo = format!("[https://github.com/{}]", pkg_spec.get_repo(&name));
        cols.push((name, ver, repo));
    }

    let lengths: Vec<_> = cols
        .iter()
        .map(|(n, v, r)| (n.len(), v.len(), r.len()))
        .collect();

    let max_len_name = lengths
        .iter()
        .max_by_key(|tup| tup.0)
        .map(|tup| tup.0)
        .unwrap();

    let max_len_ver = lengths
        .iter()
        .max_by_key(|tup| tup.1)
        .map(|tup| tup.1)
        .unwrap();

    println!(
        "{:<w_name$} {:<w_ver$} {}\n",
        "BIN",
        "VER",
        "REPO",
        w_name = max_len_name,
        w_ver = max_len_ver
    );

    for (name, ver, repo) in &cols {
        println!(
            "{:<w_name$} {:<w_ver$} {}",
            name,
            ver,
            repo,
            w_name = max_len_name,
            w_ver = max_len_ver
        );
    }
    Ok(())
}
