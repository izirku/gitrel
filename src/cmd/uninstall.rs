use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::UninstallArgs;
use crate::domain::package::{self, write_packages_file};
use crate::domain::uninstaller::uninstall as uninstall_binary;
use crate::domain::util::{bin_dir, message_fail, packages_file};

/// Uninstall installed packages
pub async fn uninstall(args: UninstallArgs) -> Result<()> {
    let packages_file = packages_file()?;
    let packages_installed = package::read_packages_file(&packages_file)?;

    if packages_installed.is_empty() {
        println!(
                "No managed installationts on this system. Use `{} install repo@[*|name|semver]...` to install package(s)",
                crate_name!(),
            );
        return Ok(());
    }

    let mut packages_to_uninstall: Vec<usize> = Vec::with_capacity(args.bin_names.len());
    let mut requested_packages: HashSet<_> = args.bin_names.iter().collect();
    for (i, pkg) in packages_installed.iter().enumerate() {
        if requested_packages.contains(&pkg.bin_name) {
            packages_to_uninstall.push(i);
            requested_packages.remove(&pkg.bin_name);
        }
    }

    if !requested_packages.is_empty() {
        for bin_name in requested_packages {
            eprintln!("package `{}` is not installed", bin_name);
        }
        return Ok(());
    }

    let default_bin_dir = bin_dir()?;
    let mut needs_save = false;
    let mut uninstalled_ct = 0;

    for i in packages_to_uninstall.iter() {
        let pkg = &packages_installed[*i];
        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message(format!("uninstalling {}", style(&pkg.bin_name).green()));
        pb.enable_steady_tick(Duration::from_millis(220));

        // either use the default path or the one specified in a package spec
        let pkg_path;
        let bin_dir = if let Some(p) = &pkg.path {
            pkg_path = std::path::PathBuf::try_from(p)?;
            pkg_path.as_path()
        } else {
            default_bin_dir.as_path()
        };

        cfg_if::cfg_if! {
            if #[cfg(target_os = "windows")] {
                let bin_name = format!("{}.exe", &pkg.bin_name);
                let bin_name = bin_name.as_ref();
            } else {
                let bin_name = &pkg.bin_name;
            }
        }

        match uninstall_binary(bin_name, bin_dir) {
            Ok(()) => {
                let msg = format!(
                    "{} uninstalled {}",
                    style('✓').green(),
                    style(&pkg.bin_name).green(),
                );
                pb.set_style(ProgressStyle::default_bar().template("{msg}").unwrap());
                pb.finish_with_message(msg);

                uninstalled_ct += 1;
                needs_save = true;
            }
            e => {
                message_fail(&pb, &pkg.bin_name, "couldn't uninstall");
                return e;
            }
        }
    }

    if needs_save {
        let mut packages_installed_new: Vec<_> =
            Vec::with_capacity(packages_installed.len() - uninstalled_ct);
        for (i, pkg) in packages_installed.into_iter().enumerate() {
            if !packages_to_uninstall.contains(&i) {
                packages_installed_new.push(pkg);
            }
        }
        write_packages_file(&packages_file, &packages_installed_new)?;
    }

    Ok(())
}
