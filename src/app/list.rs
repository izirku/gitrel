use crate::business::conf::ConfigurationManager;
use crate::error::AppError;
use crate::foundation::util::svec2_col_maj_max_lens_unchecked;
use crate::Result;
use colored::*;
use std::cmp;

/// List installed packages
pub fn process(cm: &ConfigurationManager) -> Result<()> {
    let packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => {
            println!("nothing is installed on this system");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut cols = Vec::with_capacity(packages.len());

    for (name, pkg_spec) in packages.into_iter() {
        let repo = format!("https://github.com/{}", &pkg_spec.repo);
        cols.push(vec![name, pkg_spec.requested, repo]);
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_ver$} {}",
        "Bin".green(),
        "Tag".red(),
        "Repo".blue(),
        w_name = cmp::max(3, max_lens[0]),
        w_ver = cmp::max(3, max_lens[1]),
    );

    // we have to add 4, b/c of the spaces separating columns,
    // and to factor in the 2 spaces that square brackets take up
    println!(
        "{}",
        "-".to_string()
            .repeat(cmp::max(3, max_lens[0]) + cmp::max(3, max_lens[1]) + max_lens[2] + 4)
            .yellow()
    );

    for row in &cols {
        if let [name, ver, repo] = &row[..] {
            println!(
                "{:<w_name$} {:>w_ver$} {}{}{}",
                name.green(),
                ver.red(),
                "[".cyan(),
                repo.blue(),
                "]".cyan(),
                w_name = cmp::max(3, max_lens[0]),
                w_ver = cmp::max(3, max_lens[1]),
            );
        }
    }
    Ok(())
}
