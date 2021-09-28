use crate::domain::{installer, util};
use crate::domain::util::parse_gh_repo_spec;
use crate::domain::{conf::ConfigurationManager, github::GitHub};
use crate::{AppError, Result};
use anyhow::Context;
use clap::{crate_name, ArgMatches};

/// Update installed packages
pub async fn update(matches: &ArgMatches) -> Result<()> {
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => {
            println!(
                "No managed installationts on this system. Use `{} install  repo@[*|name|semver]` to install a package",
                crate_name!(),
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut needs_save = false;
    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    // update --all packages
    if matches.is_present("all") {
        for (name, pkg) in &mut packages {
            if gh.find_match(pkg, false).await? {
                println!("updating package: {}", &name);

                gh.download(pkg, &temp_dir).await?;
                installer::install(pkg, &cm.bin_dir, cm.strip).await?;
                needs_save = true;
            }
        }
    }

    // update a single package
    if let Some(repo_spec) = matches.value_of("repo") {
        // let (_repo, repo_name, requested) = parse_gh_repo_spec(repo);
        let (repo_url, requested) = parse_gh_repo_spec(repo_spec);
        let repo_name = util::repo_name(&repo_url);
        if !packages.contains_key(&repo_name) {
            println!(
                "{1} it not installed on this system. Use `{0} install  {1}` to install a package",
                crate_name!(),
                repo_spec,
            );
            return Ok(());
        }

        let pkg = packages
            .get_mut(&repo_name)
            .context("failed to read a package spec from installed packages registry")?;

        pkg.requested = requested;

        if gh.find_match(pkg, false).await? {
            gh.download(pkg, &temp_dir).await?;
            installer::install(pkg, &cm.bin_dir, cm.strip).await?;
            needs_save = true;
        }
    }

    if needs_save {
        cm.put_packages(&packages)?;
    }

    Ok(())
}
