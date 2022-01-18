use crate::domain::github::GitHub;
use crate::domain::installer;
use crate::domain::package::Package;
use crate::domain::util;
use anyhow::{anyhow, Result};
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Install packages
pub async fn install(
    repo_spec: String,
    token: Option<&String>,
    strip: bool,
    force: bool,
) -> Result<()> {
    // let packages = Packages::new()?;
    // let mut pkgs = match packages.get() {
    //     Ok(Some(packages)) => packages,
    //     Ok(None) => PackageMap::new(),
    //     Err(e) => return Err(e),
    // };

    let mut installed = 0;

    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(token);

    let (user, repo, requested_ver) = util::parse_gh_repo_spec(&repo_spec)?;
    // let mut pkg = Package::create(repo, strip.then(|| true))?;

    // if !force && pkgs.contains_key(&repo_name) {
    //     println!(
    //         "{} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update ...` to update",
    //         &repo_name,
    //         crate_name!(),
    //         repo,
    //     );
    //     break;
    // }

    let pb = ProgressBar::new(u64::MAX);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg}")
            .progress_chars("##-"),
    );
    pb.set_message(format!("searching for {}", style(&repo).green()));
    pb.enable_steady_tick(220);

    // TODO: write util::match_asset for asset resolution
    match gh.find_new(&user, &repo, &requested_ver).await {
        Ok(Some(release)) => {
            pb.enable_steady_tick(220);
            pb.set_message(format!("downloading {}", style(&repo).green()));
            gh.download(&mut pkg, &temp_dir).await?;

            let msg = format!("installing {}", style(&repo_name).green());
            pb.set_message(msg);

            let bin_dir = util::bin_dir()?;
            match installer::install(&pkg, &bin_dir).await {
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

                    pkgs.insert(repo_name, pkg);
                    packages.put(&pkgs)?;
                    installed += 1;
                }
                Err(e) => {
                    message_fail(&pb, &repo_name, "not installed");
                    errors.push(e.context(repo_name));
                }
            }
        }
        Ok(None) => {
            message_fail(&pb, &repo_name, "not found");
        }
        Err(e) => {
            message_fail(&pb, &repo_name, "not installed");
            errors.push(e.context(repo_name));
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
