use crate::domain::conf::ConfigurationManager;
use crate::domain::github::GitHub;
use crate::domain::installer;
use crate::domain::package::{Package, PackageMap};
use crate::domain::util;
use anyhow::{anyhow, Result};
use clap::{crate_name, ArgMatches};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Install packages
pub async fn install(matches: &ArgMatches) -> Result<()> {
    // let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let force_reinstall = matches.is_present("force");
    // let repos = matches.values_of("repo").unwrap();
    let repos: Vec<&str> = matches.values_of("repo").unwrap().collect();
    let requested_ct = repos.len();
    let mut errors = Vec::with_capacity(repos.len());

    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(Some(packages)) => packages,
        Ok(None) => PackageMap::new(),
        Err(e) => return Err(e),
    };

    let mut installed = 0;

    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    for repo in repos.into_iter() {
        let mut pkg = Package::create(repo);
        let repo_name = util::repo_name(&pkg.repo);

        if !force_reinstall && packages.contains_key(&repo_name) {
            println!(
                "{} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update ...` to update",
                &repo_name,
                crate_name!(),
                repo,
            );
            break;
        }

        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}")
                .progress_chars("##-"),
        );
        pb.set_message(format!("searching for {}", style(&repo_name).green()));
        pb.enable_steady_tick(220);

        match gh.find_match(&mut pkg, force_reinstall).await {
            Ok(true) => {
                pb.set_message(format!("downloading {}", style(&repo_name).green()));
                gh.download(&mut pkg, &temp_dir).await?;

                let msg = format!("installing {}", style(&repo_name).green());
                pb.set_message(msg);
                match installer::install(&pkg, &cm.bin_dir, cm.strip).await {
                    Ok(bin_size) => {
                        let msg = format!(
                            "{} installed {} ({})",
                            style('✓').green(),
                            style(&repo_name).green(),
                            bytesize::to_string(bin_size, false),
                        );
                        pb.disable_steady_tick();
                        pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                        pb.finish_with_message(msg);

                        packages.insert(repo_name, pkg);
                        cm.put_packages(&packages)?;
                        installed += 1;
                    }
                    Err(e) => {
                        message_fail(&pb, &repo_name, "not installed");
                        errors.push(e.context(repo_name));
                    }
                }
            }
            Ok(false) => {
                message_fail(&pb, &repo_name, "not found");
            }
            Err(e) => {
                message_fail(&pb, &repo_name, "not installed");
                errors.push(e.context(repo_name));
            }
        }
    }

    println!(
        "\nInstalled {} of {} requested binaries.",
        installed, requested_ct
    );

    if errors.is_empty() {
        Ok(())
    } else {
        println!("\nsome errors has occurred during the installation:\n");
        for e in errors.iter() {
            eprintln!("{:?}\n", e);
        }

        if installed > 0 {
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
