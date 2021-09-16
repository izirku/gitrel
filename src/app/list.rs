use crate::business::data::conf::model::PackageReqMap;
use crate::business::data::conf::ConfigurationManager;
use crate::foundation::util::svec2_col_maj_max_lens_unchecked;
use anyhow::{Context, Result};
use std::fs;

/// List requested packages in a given TOML `file` file.
pub fn process(cm: &ConfigurationManager) -> Result<()> {
    let file = fs::read_to_string(cm.requested.as_path())
        .with_context(|| format!("unable to read packages file: {:?}", cm.requested))?;

    let toml = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", cm.requested))?;

    let mut cols = Vec::with_capacity(toml.len());

    for (name, pkg_spec) in toml.into_iter() {
        let pkg_spec = pkg_spec.into_detailed(&name);
        dbg!(&pkg_spec);
        let ver = format!("@ {}", &pkg_spec.matches);
        let repo = format!("[https://github.com/{}]", pkg_spec.repo.as_ref().unwrap());
        cols.push(vec![name, ver, repo]);
        dbg!(&pkg_spec);
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_ver$} {}\n",
        "BIN",
        "VER",
        "REPO",
        w_name = max_lens[0],
        w_ver = max_lens[1],
    );

    for row in &cols {
        if let [name, ver, repo] = &row[..] {
            println!(
                "{:<w_name$} {:<w_ver$} {}",
                name,
                ver,
                repo,
                w_name = max_lens[0],
                w_ver = max_lens[1],
            );
        }
    }
    Ok(())
}
