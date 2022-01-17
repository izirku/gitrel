use crate::domain::github::GitHub;
use crate::domain::packages::Packages;
use crate::domain::{installer, util};
use anyhow::{anyhow, Context, Result};
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashSet;

/// Update installed packages
// pub async fn update(only: Option<Vec<String>>) -> Result<()> {
pub async fn update(repos: Vec<String>, token: Option<&String>) -> Result<()> {
    let packages = Packages::new()?;
    let mut pkgs_installed = match packages.get() {
        Ok(Some(packages)) => packages,
        Ok(None) => {
            println!(
                "No managed installationts on this system. Use `{} install repo@[*|name|semver]...` to install a package(s)",
                crate_name!(),
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let pkgs_requested: HashSet<String> = repos.into_iter().collect();
    let temp_dir = tempfile::tempdir().context("creating a temp dir failed")?;
    let gh = GitHub::create(token);
    let mut needs_save = false;
    let mut updated = 0;
    let mut errors = Vec::with_capacity(pkgs_installed.len());

    let bin_dir = util::bin_dir()?;
    for (bin_name, pkg) in pkgs_installed.iter_mut() {
        if pkgs_requested.is_empty() || pkgs_requested.contains(bin_name) {
            let pb = ProgressBar::new(u64::MAX);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg}")
                    .progress_chars("##-"),
            );
            pb.set_message(format!("searching for {}", style(bin_name).green()));
            pb.enable_steady_tick(220);

            match gh.find_match(pkg, false).await {
                Ok(true) => {
                    pb.set_message(format!("downloading {}", style(&bin_name).green()));
                    gh.download(pkg, &temp_dir).await?;

                    let msg = format!("updating {}", style(&bin_name).green());
                    pb.set_message(msg);
                    match installer::install(pkg, bin_dir.as_path()).await {
                        Ok(bin_size) => {
                            let msg = format!(
                                "{} updated {} ({})",
                                style('✓').green(),
                                style(&bin_name).green(),
                                bytesize::to_string(bin_size, false),
                            );
                            pb.disable_steady_tick();
                            pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                            pb.set_message(msg);

                            needs_save = true;
                            updated += 1;
                        }
                        Err(e) => {
                            message_fail(&pb, bin_name, "not updated");
                            errors.push(e.context(bin_name.to_owned()));
                        }
                    }
                }
                Ok(false) => {
                    let msg = format!(
                        "{} already up to date {}",
                        style('-').green(),
                        style(&bin_name).green(),
                    );
                    pb.disable_steady_tick();
                    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                    pb.set_message(msg);
                }
                Err(e) => {
                    message_fail(&pb, bin_name, "not updated");
                    errors.push(e.context(bin_name.to_owned()));
                }
            }

            pb.finish();
        }
    }

    if needs_save {
        packages.put(&pkgs_installed)?;
    }

    let requested_tot = if pkgs_requested.is_empty() {
        pkgs_installed.len()
    } else {
        pkgs_requested.len()
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

fn message_fail(pb: &ProgressBar, item: &str, msg: &str) {
    let msg = format!("{} {} {}", style('✗').red(), msg, style(&item).red());
    pb.disable_steady_tick();
    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
    pb.finish_with_message(msg);
}
