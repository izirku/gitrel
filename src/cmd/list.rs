use crate::domain::conf::ConfigurationManager;
use crate::error::AppError;
use crate::foundation::util::svec2_col_maj_max_lens_unchecked;
use crate::Result;
use clap::ArgMatches;
use colored::*;
use std::cmp;

/// List installed packages
pub fn list(matches: &ArgMatches) -> Result<()> {
    let cm = ConfigurationManager::with_clap_matches(matches)?;
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
        cols.push(vec![
            name,
            pkg_spec.requested,
            pkg_spec.tag.unwrap_or_else(|| "".to_string()),
            repo,
        ]);
    }

    let max_lens = svec2_col_maj_max_lens_unchecked(&cols);

    println!(
        "{:<w_name$} {:<w_req$} {:<w_inst$} {}",
        "Bin".green(),
        "Requested".red(),
        "Installed".cyan(),
        "Repo".blue(),
        w_name = cmp::max(3, max_lens[0]),
        w_req = cmp::max(9, max_lens[1]),
        w_inst = cmp::max(9, max_lens[2]),
    );

    // we have to add 5, b/c of the spaces separating columns,
    // and to factor in the 2 spaces that square brackets take up
    println!(
        "{}",
        "-".to_string()
            .repeat(
                cmp::max(3, max_lens[0])
                    + cmp::max(9, max_lens[1])
                    + cmp::max(9, max_lens[2])
                    + max_lens[3]
                    + 5
            )
            .yellow()
    );

    for row in &cols {
        if let [name, requested, installed, repo] = &row[..] {
            println!(
                "{:<w_name$} {:>w_req$} {:>w_inst$} {}{}{}",
                name.green(),
                requested.red(),
                installed.cyan(),
                "[".cyan(),
                repo.blue(),
                "]".cyan(),
                w_name = cmp::max(3, max_lens[0]),
                w_req = cmp::max(9, max_lens[1]),
                w_inst = cmp::max(9, max_lens[2]),
            );
        }
    }
    Ok(())
}
