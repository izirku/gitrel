use std::io::ErrorKind;

use crate::domain::package::Package;
use crate::domain::util::{self, message_fail};
use crate::domain::{github::GitHub, util::packages_file};
use crate::domain::{installer, package};
use anyhow::{anyhow, Result};
use clap::crate_name;
use console::style;
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};

/// Install packages command
pub async fn install(
    repo_spec: String,
    token: Option<&String>,
    strip: bool,
    force: bool,
) -> Result<()> {
    let packages_file = packages_file()?;
    let mut packages_installed = package::read_packages_file(&packages_file)?;
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");
    let gh = GitHub::create(token);
    let (user, repo, requested_ver) = util::parse_gh_repo_spec(&repo_spec)?;

    let mut already_installed = None;
    for (i, package) in packages_installed.iter().enumerate() {
        if package.repo == repo {
            already_installed = Some(i);
            break;
        }
    }

    if !force && already_installed.is_some() {
        println!(
                "{0} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update {2}` to update",
                &repo,
                crate_name!(),
                &repo_spec
            );
        return Ok(());
    }

    let mut pb = ProgressBar::new(u64::MAX);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg}")
            .progress_chars("##-"),
    );
    pb.set_message(format!("searching for {}", style(&repo).green()));
    pb.enable_steady_tick(220);

    // TODO: maybe write util::match_asset for asset resolution
    match gh.find_new(&user, &repo, &requested_ver).await {
        Ok(Some(release)) => {
            pb.enable_steady_tick(220);
            // pb.set_message(format!("downloading {}", style(&repo).green()));

            let (asset_id, asset_name) = if release.assets.len() == 1 {
                (release.assets[0].id, release.assets[0].name.as_str())
            } else {
                pb.disable_steady_tick();
                // pb.finish_and_clear();

                let selection: Vec<_> = release.assets.iter().map(|asset| &asset.name).collect();
                // dbg!(selection);
                let res = Select::with_theme(&ColorfulTheme::default())
                    // .with_prompt(format!(
                    //     "downloading `{}`, multiple assets found, please select one:",
                    //     &repo
                    // ))
                    .items(&selection)
                    .interact();
                // .map_err(|_| anyhow::Error::msg("selection interrupted, aborting..."))?;

                match res {
                    Ok(i) => {
                        // pb = ProgressBar::new(u64::MAX);
                        // pb.set_style(
                        //     ProgressStyle::default_bar()
                        //         .template("{spinner:.green} {msg}")
                        //         .progress_chars("##-"),
                        // );
                        // pb.set_message(format!("downloading {}", style(&repo).green()));
                        pb.enable_steady_tick(220);
                        (release.assets[i].id, release.assets[i].name.as_str())
                    }
                    Err(e) if e.kind() == ErrorKind::Interrupted => return Ok(()),
                    Err(e) => return Err(anyhow::Error::msg(e)),
                }
                // dbg!(i);
            };

            let asset_path = gh
                .download(&user, &repo, asset_id, asset_name, &temp_dir)
                .await?;

            let msg = format!("installing {}", style(&repo).green());
            pb.set_message(msg);

            let bin_dir = util::bin_dir()?;
            let bin_name = repo.to_lowercase();

            let res = installer::install(asset_name, &asset_path, &bin_dir, &bin_name, strip).await;

            match res {
                Ok(bin_size) => {
                    let msg = format!(
                        "{} installed {} ({})",
                        style('✓').green(),
                        style(&repo).green(),
                        bytesize::to_string(bin_size, false),
                    );
                    pb.disable_steady_tick();
                    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                    pb.finish_with_message(msg);

                    let package = Package {
                        user,
                        repo,
                        tag: release.tag_name,
                        bin_name,
                        requested: requested_ver,
                        strip: strip.then(|| true),
                        timestamp: release.published_at,
                    };

                    if let Some(i) = already_installed {
                        dbg!(i);
                        packages_installed[i] = package;
                    } else {
                        packages_installed.push(package);
                    }

                    package::write_packages_file(&packages_file, &packages_installed)?;
                    Ok(())
                }
                Err(e) => {
                    message_fail(&pb, &repo, "not installed");
                    eprintln!("{:?}\n", e);
                    Err(anyhow!("operation failed"))
                }
            }
        }
        Ok(None) => {
            message_fail(&pb, &repo, "not found");
            Ok(())
        }
        Err(e) => {
            message_fail(&pb, &repo, "not installed");
            eprintln!("{:?}\n", e);
            Err(anyhow!("operation failed"))
        }
    }
}
