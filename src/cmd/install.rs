use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::InstallArgs;
use crate::domain::package::Package;
use crate::domain::util::{self, message_fail};
use crate::domain::{executor, installer, package};
use crate::domain::{github::GitHub, util::packages_file};

/// Install packages command
pub async fn install(args: InstallArgs) -> Result<()> {
    let packages_file = packages_file()?;
    let mut packages_installed = package::read_packages_file(&packages_file)?;
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");
    let gh = GitHub::create(args.token.as_ref());
    let (user, repo, requested_ver) = util::parse_gh_repo_spec(&args.repo_spec)?;

    let mut already_installed = None;
    for (i, package) in packages_installed.iter().enumerate() {
        if package.repo == repo {
            already_installed = Some(i);
            break;
        }
    }

    if !args.force && already_installed.is_some() {
        println!(
                "{0} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update {2}` to update",
                &repo,
                crate_name!(),
                &args.repo_spec
            );
        return Ok(());
    }

    let pb = ProgressBar::new(u64::MAX);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_message(format!("searching for {}", style(&repo).green()));
    pb.enable_steady_tick(Duration::from_millis(220));

    match gh
        .find_new(
            &user,
            &repo,
            &requested_ver,
            args.asset_glob.as_deref(),
            args.asset_re.as_deref(),
        )
        .await
    {
        Ok(release) => {
            let (asset_id, asset_name) = (release.assets[0].id, release.assets[0].name.as_str());

            pb.set_message(format!("downloading {}", style(&repo).green()));
            let asset_path = gh
                .download(&user, &repo, asset_id, asset_name, &temp_dir)
                .await?;

            let msg = format!("installing {}", style(&repo).green());
            pb.set_message(msg);

            // let bin_dir = util::bin_dir()?;
            let bin_dir = if let Some(p) = &args.path {
                PathBuf::try_from(p)?
            } else {
                util::bin_dir()?
            };

            let bin_name = if let Some(new_name) = args.rename_binary.to_owned() {
                new_name
            } else {
                repo.to_lowercase()
            };

            #[cfg(not(target_os = "windows"))]
            let res = installer::install(
                asset_name,
                &asset_path,
                &bin_dir,
                &bin_name,
                args.strip,
                args.entry_glob.as_deref(),
                args.entry_re.as_deref(),
            )
            .await;

            #[cfg(target_os = "windows")]
            let res = installer::install(
                asset_name,
                &asset_path,
                &bin_dir,
                &bin_name,
                args.entry_glob.as_deref(),
                args.entry_re.as_deref(),
            )
            .await;

            match res {
                Ok(bin_size) => {
                    #[cfg(not(target_os = "windows"))]
                    if let Some(cmd) = &args.cmd_after {
                        let bin_path = bin_dir.join(&bin_name);
                        executor::exec(&bin_path, cmd)?;
                    }

                    let msg = format!(
                        "{} installed {} ({})",
                        style('✓').green(),
                        style(&repo).green(),
                        bytesize::to_string(bin_size, false),
                    );
                    pb.set_style(ProgressStyle::default_bar().template("{msg}").unwrap());
                    pb.finish_with_message(msg);

                    #[cfg(not(target_os = "windows"))]
                    let package = Package {
                        user,
                        repo,
                        bin_name,
                        path: args.path,
                        tag: release.tag_name,
                        requested: requested_ver,
                        strip: args.strip.then(|| true),
                        timestamp: release.published_at,
                        asset_glob: args.asset_glob,
                        asset_re: args.asset_re,
                        entry_glob: args.entry_glob,
                        entry_re: args.entry_re,
                        cmd_after: args.cmd_after,
                    };

                    #[cfg(target_os = "windows")]
                    let package = Package {
                        user,
                        repo,
                        bin_name,
                        path: args.path,
                        tag: release.tag_name,
                        requested: requested_ver,
                        timestamp: release.published_at,
                        asset_glob: args.asset_glob,
                        asset_re: args.asset_re,
                        entry_glob: args.entry_glob,
                        entry_re: args.entry_re,
                    };

                    if let Some(i) = already_installed {
                        packages_installed[i] = package;
                    } else {
                        packages_installed.push(package);
                    }

                    // sort installed packages before writing them out, so that the subsequent `list`
                    // and `update` commands display & iterate over installed packages in an alphabetic order
                    packages_installed.sort_unstable_by(|a, b| a.bin_name.cmp(&b.bin_name));

                    package::write_packages_file(&packages_file, &packages_installed)?;
                    Ok(())
                }
                Err(e) => {
                    message_fail(&pb, &repo, "not installed");

                    use crate::domain::error::InstallerError;
                    match e {
                        InstallerError::AnyHow(e) => Err(e),
                        e => {
                            eprint!("\nreason: {}\n\n", e);
                            Ok(())
                        }
                    }
                }
            }
        }
        Err(e) => {
            message_fail(&pb, &repo, "not installed");

            use crate::domain::error::GithubError;
            match e {
                GithubError::AnyHow(e) => Err(e),
                e => {
                    eprint!("\nreason: {}\n\n", e);
                    Ok(())
                }
            }
        }
    }
}
