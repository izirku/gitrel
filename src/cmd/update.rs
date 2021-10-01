use crate::domain::package::Package;
use crate::domain::{conf::ConfigurationManager, github::GitHub};
use crate::domain::{installer, util};
use anyhow::{anyhow, Context, Result};
use clap::{crate_name, ArgMatches};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::BTreeMap;

/// Update installed packages
pub async fn update(matches: &ArgMatches) -> Result<()> {
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut pkgs_installed = match cm.get_packages() {
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

    let repos: Vec<&str> = if let Some(repos) = matches.values_of("repo") {
        repos.collect()
    } else {
        vec![]
    };

    let mut pkgs_requested: BTreeMap<String, Package> = BTreeMap::new();
    for repo in repos {
        let pkg = Package::create(repo);
        let repo_name = util::repo_name(&pkg.repo);
        pkgs_requested.insert(repo_name, pkg);
    }

    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().context("creating a temp dir failed")?;

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);
    let mut needs_save = false;
    let mut updated = 0;
    let mut errors = Vec::with_capacity(pkgs_installed.len());

    for (repo_name, pkg) in pkgs_installed.iter_mut() {
        if matches.is_present("all") || pkgs_requested.contains_key(repo_name) {
            let pb = ProgressBar::new(u64::MAX);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg}")
                    .progress_chars("##-"),
            );
            pb.set_message(format!("searching for {}", style(repo_name).green()));
            pb.enable_steady_tick(220);

            match gh.find_match(pkg, false).await {
                Ok(true) => {
                    pb.set_message(format!("downloading {}", style(&repo_name).green()));
                    gh.download(pkg, &temp_dir).await?;

                    let msg = format!("updating {}", style(&repo_name).green());
                    pb.set_message(msg);
                    match installer::install(pkg, &cm.bin_dir, cm.strip).await {
                        Ok(bin_size) => {
                            let msg = format!(
                                "{} updating {} ({})",
                                style('✓').green(),
                                style(&repo_name).green(),
                                bytesize::to_string(bin_size, false),
                            );
                            pb.disable_steady_tick();
                            pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                            pb.finish_with_message(msg);

                            needs_save = true;
                            updated += 1;
                        }
                        Err(e) => {
                            message_fail(&pb, repo_name, "not updated");
                            errors.push(e.context(repo_name.to_owned()));
                        }
                    }
                }
                Ok(false) => {
                    let msg = format!(
                        "{} already up to date {}",
                        style('-').green(),
                        style(&repo_name).green(),
                    );
                    pb.disable_steady_tick();
                    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                    pb.finish_with_message(msg);
                }
                Err(e) => {
                    message_fail(&pb, repo_name, "not updated");
                    errors.push(e.context(repo_name.to_owned()));
                }
            }
        }
    }

    if needs_save {
        cm.put_packages(&pkgs_installed)?;
    }

    println!("\nUpdated {} of {} binaries.", updated, pkgs_installed.len());

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

fn message_fail(pb: &ProgressBar, repo_name: &str, msg: &str) {
    let msg = format!("{} {} {}", style('✗').red(), msg, style(&repo_name).red());
    pb.disable_steady_tick();
    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
    pb.finish_with_message(msg);
}
