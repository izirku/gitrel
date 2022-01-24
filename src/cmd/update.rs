use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::UpdateArgs;
use crate::domain::error::GithubError;
use crate::domain::github::GitHub;
use crate::domain::package;
use crate::domain::util::packages_file;
use crate::domain::{installer, util};

/// Update installed packages
pub async fn update(args: UpdateArgs) -> Result<()> {
    let packages_file = packages_file()?;
    let mut packages_installed = package::read_packages_file(&packages_file)?;

    if packages_installed.is_empty() {
        println!(
                "No managed installationts on this system. Use `{} install repo@[*|name|semver]...` to install package(s)",
                crate_name!(),
            );
        return Ok(());
    }

    // determine what we need to try and update
    let packages_to_update: HashSet<_> = if args.bin_names.is_empty() {
        packages_installed
            .iter()
            .enumerate()
            .map(|(i, _)| i)
            .collect()
    } else {
        let mut requested_packages: HashSet<_> = args.bin_names.iter().collect();
        let packages_to_update = packages_installed
            .iter()
            .enumerate()
            .filter_map(|(i, pkg)| {
                if requested_packages.contains(&pkg.bin_name) {
                    requested_packages.remove(&pkg.bin_name);
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // bail if we have requested bins to update, that are not installed
        if !requested_packages.is_empty() {
            for bin_name in requested_packages {
                eprintln!("\npackage `{}` is not installed", bin_name);
            }
            return Ok(());
        }

        packages_to_update
    };

    let gh = GitHub::create(args.token.as_ref());
    let temp_dir = tempfile::tempdir().context("creating a temp dir failed")?;
    let bin_dir = util::bin_dir()?;
    let mut needs_save = false;
    let mut updated = 0;

    for i in packages_to_update {
        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}")
                .progress_chars("##-"),
        );
        pb.set_message(format!(
            "searching for {}",
            style(&packages_installed[i].bin_name).green()
        ));
        pb.enable_steady_tick(220);

        match gh.find_existing(&packages_installed[i]).await {
            Ok(release) => {
                pb.set_message(format!(
                    "downloading {}",
                    style(&packages_installed[i].bin_name).green()
                ));

                let asset_path = gh
                    .download(
                        &packages_installed[i].user,
                        &packages_installed[i].repo,
                        release.assets[0].id,
                        &release.assets[0].name,
                        &temp_dir,
                    )
                    .await?;

                pb.set_message(format!(
                    "updating {}",
                    style(&packages_installed[i].bin_name).green()
                ));
                let res = installer::install(
                    &packages_installed[i].repo.to_lowercase(),
                    &release.assets[0].name,
                    &asset_path,
                    bin_dir.as_path(),
                    &packages_installed[i].bin_name,
                    packages_installed[i].strip.unwrap_or_default(),
                    packages_installed[i].asset_glob.as_deref(),
                    packages_installed[i].asset_re.as_deref(),
                )
                .await;

                match res {
                    Ok(bin_size) => {
                        packages_installed[i].tag = release.tag_name;
                        packages_installed[i].timestamp = release.published_at;

                        let msg = format!(
                            "{} updated {} ({})",
                            style('✓').green(),
                            style(&packages_installed[i].bin_name).green(),
                            bytesize::to_string(bin_size, false),
                        );
                        pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                        pb.finish_with_message(msg);

                        needs_save = true;
                        updated += 1;
                    }
                    Err(e) => {
                        util::message_fail(&pb, &packages_installed[i].bin_name, "not updated");

                        use crate::domain::error::InstallerError;
                        match e {
                            InstallerError::AnyHow(e) => {
                                return Err(e);
                            }
                            e => {
                                eprint!("\nreason: {}\n\n", e);
                                return Ok(());
                            }
                        }
                    }
                }
            }
            Err(GithubError::AlreadyUpToDate) => {
                let msg = format!(
                    "{} already up to date {}",
                    style('✓').green(),
                    style(&packages_installed[i].bin_name).green(),
                );
                pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                pb.finish_with_message(msg);
            }
            Err(e) => {
                util::message_fail(&pb, &packages_installed[i].bin_name, "failed to update");

                match e {
                    GithubError::AnyHow(e) => {
                        return Err(e);
                    }
                    e => {
                        eprint!("\nreason: {}\n\n", e);
                        return Ok(());
                    }
                }
            }
        }
    }

    if needs_save {
        package::write_packages_file(&packages_file, &packages_installed)?;
    }

    let requested_tot = if args.bin_names.is_empty() {
        packages_installed.len()
    } else {
        args.bin_names.len()
    };
    println!("\nUpdated {} of {} binaries.", updated, requested_tot);

    Ok(())
}
