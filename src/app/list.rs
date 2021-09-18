use anyhow::Result;

use crate::{
    business::{conf::ConfigurationManager, util::parse_gh_repo_name},
    foundation::util::svec2_col_maj_max_lens_unchecked,
};

/// List requested packages
pub fn process(cm: &ConfigurationManager) -> Result<()> {
    use crate::business::conf::RequestedPackage::{Detailed, Simple};

    let req_pkgs = cm.requested_packages()?;

    let mut cols = Vec::with_capacity(req_pkgs.len());

    for (name, pkg_spec) in req_pkgs.into_iter() {
        match pkg_spec {
            Simple(tag) => {
                let repo = format!("[https://github.com/{}]", parse_gh_repo_name(&name));
                cols.push(vec![name, tag, repo]);
            }
            Detailed(details) => {
                let repo = format!(
                    "[https://github.com/{}]",
                    details.repo.unwrap_or(parse_gh_repo_name(&name))
                );
                cols.push(vec![name, details.matches, repo]);
            }
        }
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_ver$} {}\n",
        "BIN",
        "TAG",
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
