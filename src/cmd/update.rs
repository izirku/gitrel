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
//         if matches.is_present("all") || pkgs_requested.contains_key(repo_name) {

//             match gh.find_match(pkg, false).await {
//                 Ok(true) => {
//                     pb.set_message(format!("downloading {}", style(&repo_name).green()));
//                     gh.download(pkg, &temp_dir).await?;

//                     let msg = format!("installing {}", style(&repo_name).green());
//                     pb.set_message(msg);
//                     match installer::install(&pkg, &cm.bin_dir, cm.strip).await {
//                         Ok(bin_size) => {
//                             let msg = format!(
//                                 "{} installed {} ({})",
//                                 style('✓').green(),
//                                 style(&repo_name).green(),
//                                 bytesize::to_string(bin_size, false),
//                             );
//                             pb.disable_steady_tick();
//                             pb.set_style(ProgressStyle::default_bar().template("{msg}"));
//                             pb.finish_with_message(msg);

//                             // packages.insert(repo_name, pkg);
//                             // cm.put_packages(&packages)?;
//                             needs_save = true;
//                             installed += 1;
//                         }
//                         Err(e) => {
//                             message_fail(&pb, &repo_name, "not updated");
//                             errors.push(e.context(repo_name));
//                         }
//                     }
//                 }
//                 Ok(false) => {
//                     message_fail(&pb, &repo_name, "not found");
//                 }
//                 Err(e) => {
//                     message_fail(&pb, &repo_name, "not updated");
//                     errors.push(e.context(repo_name));
//                 }
//             }
//             //             if gh.find_match(pkg, false).await? {
//             //                 // println!("updating package: {}", &repo_name);

//             //                 pb.set_message(format!("downloading {}", style(&repo_name).green()));
//             //                 gh.download(pkg, &temp_dir).await?;

//             //                 let msg = format!("updating {}", style(&repo_name).green());
//             //                 pb.set_message(msg);
//             //                 installer::install(pkg, &cm.bin_dir, cm.strip).await?;

//             //                 needs_save = true;
//             //             }
//         }
//     }
//     // update a single package
//     //     if let Some(repo_spec) = matches.value_of("repo") {
//     //         // let (_repo, repo_name, requested) = parse_gh_repo_spec(repo);
//     //         let (repo_url, requested) = parse_gh_repo_spec(repo_spec);
//     //         let repo_name = util::repo_name(&repo_url);
//     //         if !pkgs_installed.contains_key(&repo_name) {
//     //             println!(
//     //                 "{1} it not installed on this system. Use `{0} install  {1}` to install a package",
//     //                 crate_name!(),
//     //                 repo_spec,
//     //             );
//     //             return Ok(());
//     //         }

//     //         let pkg = pkgs_installed
//     //             .get_mut(&repo_name)
//     //             .context("failed to read a package spec from installed packages registry")?;

//     //         pkg.requested = requested;

//     //         if gh.find_match(pkg, false).await? {
//     //             // gh.download(&pb, pkg, &temp_dir).await?;
//     //             gh.download(pkg, &temp_dir).await?;
//     //             installer::install(pkg, &cm.bin_dir, cm.strip).await?;
//     //             needs_save = true;
//     //         }
//     //     }

//     // if needs_save {
//     //     cm.put_packages(&pkgs_installed)?;
//     // }
// let mut pkgs_updated: BTreeMap<String, Package> = BTreeMap::new();
// for (repo_name, pkg) in pkgs_installed.into_iter() {

// if matches.is_present("all") ||
//    }
// let mut pkgs_requested: BTreeMap<String, >;
// if matches.is_present("all") {
//     pkgs_requested = pkgs_installed;
// } else {
//     for repo_spec in repos.into_iter() {
//         let (repo_url, requested) = parse_gh_repo_spec(repo_spec);
//         let repo_name = util::repo_name(&repo_url);
//         if pkgs_installed.contains_key(&repo_name) {
//             let pkg_installed = pkgs_installed.get_mut(&repo_name).unwrap();
//             let pkg = Package {
//                 repo: repo_url,
//                 tag: pkg_installed.tag.clone(),
//                 requested,
//                 timestamp: pkg_installed.timestamp,
//                 asset_id: None,
//                 asset_name: None,
//                 asset_path: None,
//             };
//             pkgs_requested.insert(repo_name, pkg);
//         }
//     }
// }
// let pb = ProgressBar::new(100);
// pb.set_style(
//         ProgressStyle::default_bar()
//             // .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
//             .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
//             .progress_chars("#>-")
//     );
