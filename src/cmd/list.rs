use anyhow::Result;
use clap::crate_name;
use console::style;
use tabled::{Alignment, Column, Disable, Format, Modify, Object, Row, Style, Table, Tabled};

use crate::{
    cli::ListArgs,
    domain::{
        package,
        util::{self, packages_file},
    },
};

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
    #[header("Path")]
    path: &'a str,
}

/// List installed packages
pub fn list(args: ListArgs) -> Result<()> {
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

    let default_bin_path = util::bin_dir_display()?;

    for pkg in &packages_installed {
        list_lines.push(ListLine {
            bin: &pkg.bin_name,
            requested: &pkg.requested,
            installed: &pkg.tag,
            repository: format!("https://github.com/{}/{}", &pkg.user, &pkg.repo),
            path: pkg.path.as_ref().map_or(default_bin_path, |p| p.as_str()),
        });
    }

    let table = create_table(&list_lines, args.wide);

    println!("\n{}", table);

    Ok(())
}

fn create_table(data: &[ListLine], wide: bool) -> Table {
    // let table = Table::new(&list_lines).with(Style::blank().header('-').header_intersection('+'))
    let table = Table::new(data)
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
            Modify::new(Column(3..4))
                .with(Alignment::left())
                .with(Format(|s| style(s).blue().to_string())),
        )
        .with(Modify::new(Column(3..4).not(Row(..1))).with(Format(|s| {
            format!("{}{}{}", style('[').cyan(), s, style(']').cyan())
        })))
        .with(
            Modify::new(Column(4..))
                .with(Alignment::left())
                .with(Format(|s| style(s).green().to_string())),
        );

    if !wide {
        table.with(Disable::Column(4..))
    } else {
        table
    }
    .with(
        Style::modern()
            .horizontal_off()
            .top_off()
            .bottom_off()
            .left_off()
            .right_off(),
    )
}
