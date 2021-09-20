use crate::business::conf::ConfigurationManager;
use crate::foundation::util::svec2_col_maj_max_lens_unchecked;
use anyhow::Result;

/// List requested packages
pub fn process(cm: &ConfigurationManager) -> Result<()> {
    let req_pkgs = cm.requested_packages()?;
    let mut cols = Vec::with_capacity(req_pkgs.len());

    for (name, pkg_spec) in req_pkgs.into_iter() {
        let repo = format!("[https://github.com/{}]", &pkg_spec.repo);
        cols.push(vec![name, pkg_spec.version, repo]);
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_ver$} REPO\n",
        "BIN",
        "TAG",
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
