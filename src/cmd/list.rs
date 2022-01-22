use anyhow::Result;
use clap::crate_name;
use console::style;
use tabled::{style::Line, Alignment, Column, Format, Modify, Object, Row, Style, Table, Tabled};

use crate::domain::{package, util::packages_file};

#[derive(Tabled)]
struct ListLine<'a> {
    #[header("Bin")]
    bin: &'a str,
    #[header("Requested")]
    requested: &'a str,
    #[header("Installed")]
    installed: &'a str,
    #[header("Repository")]
    repository: String,
}

/// List installed packages
pub fn list() -> Result<()> {
    let packages_file = packages_file()?;
    let packages_installed = package::read_packages_file(&packages_file)?;

    if packages_installed.is_empty() {
        println!(
                "No managed installationts on this system. Use `{} install repo@[*|name|semver]...` to install package(s)",
                crate_name!(),
            );
        return Ok(());
    }

    let mut list_lines = Vec::with_capacity(packages_installed.len());

    for pkg in &packages_installed {
        list_lines.push(ListLine {
            bin: &pkg.bin_name,
            requested: &pkg.requested,
            installed: &pkg.tag,
            repository: format!("https://github.com/{}/{}", &pkg.user, &pkg.repo),
        });
    }

    let table = Table::new(&list_lines)
        .with(Style::NO_BORDER.header(Some(Line::short('-', '+'))))
        .with(
            Modify::new(Column(..1))
                .with(Alignment::left())
                .with(Format(|s| style(s).green().to_string())),
        )
        .with(
            Modify::new(Column(1..2))
                .with(Alignment::right())
                .with(Format(|s| style(s).red().to_string())),
        )
        .with(
            Modify::new(Column(2..3))
                .with(Alignment::right())
                .with(Format(|s| style(s).cyan().to_string())),
        )
        .with(
            Modify::new(Column(3..))
                .with(Alignment::left())
                .with(Format(|s| style(s).blue().to_string())),
        )
        .with(Modify::new(Column(3..).not(Row(..1))).with(Format(|s| {
            format!("{}{}{}", style('[').cyan(), s, style(']').cyan())
        })));

    println!("{}", table);

    Ok(())
}
