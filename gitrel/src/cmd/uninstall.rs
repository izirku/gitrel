use crate::domain::packages::Packages;
use crate::domain::uninstaller::uninstall as uninstall_binary;
use crate::domain::util::{bin_dir, message_fail};
use anyhow::{anyhow, Result};
use clap::crate_name;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};

/// Uninstall installed packages
pub async fn uninstall(bins_to_uninstall: Vec<String>) -> Result<()> {
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

    let bin_dir = bin_dir()?;
    let mut needs_save = false;
    let mut uninstalled_ct = 0;
    let mut errors = Vec::with_capacity(bins_to_uninstall.len());

    for bin_name in &bins_to_uninstall {
        let pb = ProgressBar::new(u64::MAX);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg}")
                .progress_chars("##-"),
        );
        pb.set_message(format!("uninstalling {}", style(&bin_name).green()));
        pb.enable_steady_tick(220);

        if pkgs_installed.contains_key(bin_name) {
            match uninstall_binary(bin_name, bin_dir.as_path()) {
                Ok(()) => {
                    pkgs_installed.remove(bin_name);
                    let msg = format!(
                        "{} uninstalled {}",
                        style('✓').green(),
                        style(&bin_name).green(),
                    );
                    pb.disable_steady_tick();
                    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
                    pb.finish_with_message(msg);

                    uninstalled_ct += 1;
                    needs_save = true;
                }
                Err(e) => {
                    message_fail(&pb, bin_name, "couldn't uninstall");
                    errors.push(e.context(bin_name.to_owned()));
                }
            }
        } else {
            message_fail(&pb, bin_name, "is not installed");
        }
    }

    if needs_save {
        packages.put(&pkgs_installed)?;
    }

    println!("\nUninstalled {}.", uninstalled_ct);

    if errors.is_empty() {
        Ok(())
    } else {
        println!("\nsome errors has occurred during the uninstall process:\n");
        for e in errors.iter() {
            eprintln!("{:?}\n", e);
        }

        if uninstalled_ct > 0 {
            Err(anyhow!("partial success"))
        } else {
            Err(anyhow!("operation failed"))
        }
    }
}
