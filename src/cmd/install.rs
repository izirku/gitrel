use crate::domain::conf::ConfigurationManager;
use crate::domain::github::GitHub;
use crate::domain::installer;
use crate::domain::package::{Package, PackageMap};
use crate::domain::util;
use crate::{AppError, Result};
use clap::{crate_name, ArgMatches};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

// Install packages
pub async fn install(matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);
    // dbg!(&pkg);
    // let repo_name = (&pkg.repo.path()[1..]).to_lowercase();
    let repo_name = util::repo_name(&pkg.repo);
    // dbg!(&repo_name);
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => PackageMap::new(),
        Err(e) => return Err(e),
    };

    let force_reinstall = matches.is_present("force");
    if !force_reinstall && packages.contains_key(&repo_name) {
        println!(
                "{} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update ...` to update",
                &repo_name,
                crate_name!(),
                repo,
            );
        return Ok(());
    }

    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    if gh.find_match(&mut pkg, force_reinstall).await? {
        let repo_name = util::repo_name(&pkg.repo);

        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}\n[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .progress_chars("#>-")
        );

        let msg = format!("downloading: {}", style(&repo_name).green());
        pb.set_message(msg);
        gh.download(&pb, &mut pkg, &temp_dir).await?;

        let msg = format!("installing: {}", style(&repo_name).green());
        pb.set_message(msg);
        match installer::install(&pkg, &cm.bin_dir, cm.strip).await {
            Ok(bin_size) => {
                let msg = format!(
                    "{} installed: {} ({})",
                    style('✓').green(),
                    style(&repo_name).green(),
                    bytesize::to_string(bin_size, false),
                );
                pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                pb.finish_with_message(msg);
            }
            Err(e) => {
                let msg = format!(
                    "{} installed: {}",
                    style('✗').red(),
                    style(&repo_name).red()
                );
                pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                pb.finish_with_message(msg);
                // TODO: proper error aggregation and reporting?
                return Err(e);
            }
        }
        // println!("size: {}", bytesize::to_string(bin_size, false));

        packages.insert(repo_name, pkg);
        cm.put_packages(&packages)?;
    }
    Ok(())
}
