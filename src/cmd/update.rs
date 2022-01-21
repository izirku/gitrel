use std::collections::HashSet;

use anyhow::{anyhow, Context, Result};
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::UpdateArgs;
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
                eprintln!("package `{}` is not installed", bin_name);
            }
            return Err(anyhow!("operation failed"));
        }

        packages_to_update
    };

    let gh = GitHub::create(args.token.as_ref());
    let temp_dir = tempfile::tempdir().context("creating a temp dir failed")?;
    let bin_dir = util::bin_dir()?;
    let mut needs_save = false;
    let mut updated = 0;
    let mut errors = Vec::with_capacity(packages_to_update.len());

    for i in packages_to_update {
        let pkg = &mut packages_installed[i];

        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}")
                .progress_chars("##-"),
        );
        pb.set_message(format!("searching for {}", style(&pkg.bin_name).green()));
        pb.enable_steady_tick(220);

        match gh.find_existing(pkg).await {
            Ok(Some(release)) => {
                pb.set_message(format!("downloading {}", style(&pkg.bin_name).green()));

                let asset_path = gh
                    .download(
                        &pkg.user,
                        &pkg.repo,
                        release.assets[0].id,
                        &release.assets[0].name,
                        &temp_dir,
                    )
                    .await?;

                pb.set_message(format!("updating {}", style(&pkg.bin_name).green()));
                let res = installer::install(
                    &release.assets[0].name,
                    &asset_path,
                    bin_dir.as_path(),
                    &pkg.bin_name,
                    pkg.strip.unwrap_or_default(),
                    pkg.asset_contains.as_deref(),
                    pkg.asset_re.as_deref(),
                )
                .await;

                match res {
                    Ok(bin_size) => {
                        pkg.tag = release.tag_name;
                        pkg.timestamp = release.published_at;

                        let msg = format!(
                            "{} updated {} ({})",
                            style('✓').green(),
                            style(&pkg.bin_name).green(),
                            bytesize::to_string(bin_size, false),
                        );
                        // pb.disable_steady_tick();
                        pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                        pb.finish_with_message(msg);

                        needs_save = true;
                        updated += 1;
                    }
                    Err(e) => {
                        util::message_fail(&pb, &pkg.bin_name, "not updated");
                        errors.push(e.context(pkg.bin_name.to_owned()));
                    }
                }
            }
            Ok(None) => {
                let msg = format!(
                    "{} already up to date {}",
                    style('-').green(),
                    style(&pkg.bin_name).green(),
                );
                // pb.disable_steady_tick();
                pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                pb.finish_with_message(msg);
            }
            Err(e) => {
                util::message_fail(&pb, &pkg.bin_name, "not updated");
                errors.push(e.context(pkg.bin_name.to_owned()));
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

    if errors.is_empty() {
        Ok(())
    } else {
        println!("\nsome errors has occurred during the update:\n");
        for e in errors.iter() {
            eprintln!("{:?}\n", e);
        }

        if updated > 0 {
            Err(anyhow!("partial success"))
        } else {
            Err(anyhow!("operation failed"))
        }
    }
}
