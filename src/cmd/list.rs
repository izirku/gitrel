use anyhow::Result;
use clap::crate_name;
use console::style;
use owo_colors::OwoColorize;
use tabled::{
    object::{Columns, Object, Rows},
    style::Style,
    Alignment, Disable, ModifyObject, Table, Tabled,
};

use crate::{
    cli::ListArgs,
    domain::{
        package,
        util::{self, packages_file},
    },
};

#[derive(Tabled)]
struct ListLine<'a> {
    #[tabled(rename = "Bin")]
    bin: &'a str,
    #[tabled(rename = "Requested")]
    requested: &'a str,
    #[tabled(rename = "Installed")]
    installed: &'a str,
    #[tabled(rename = "Repository")]
    repository: String,
    #[tabled(rename = "Path")]
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
    let st_blue = |s: &str| s.blue().to_string();
    let st_blue_with_brackets =
        |s: &str| format!("{}{}{}", style('[').cyan(), s.blue(), style(']').cyan());
    let st_cyan = |s: &str| s.cyan().to_string();
    let st_green = |s: &str| s.green().to_string();
    let st_red = |s: &str| s.red().to_string();

    let theme = Style::modern()
        .off_top()
        .off_bottom()
        .off_horizontal()
        // NB: order matters, make sure `.lines` is before `off_left|off_right`
        .lines([(1, Style::modern().get_horizontal())])
        .off_left()
        .off_right();

    let table = Table::new(data)
        .with(Columns::single(0).modify().with(st_green))
        .with(
            Columns::single(1)
                .modify()
                .with(st_red)
                .with(Alignment::right()),
        )
        .with(
            Columns::single(2)
                .modify()
                .with(st_cyan)
                .with(Alignment::right()),
        )
        .with(Columns::single(3).modify().with(st_blue))
        .with(
            Columns::single(3)
                .not(Rows::first())
                .modify()
                .with(st_blue_with_brackets),
        )
        .with(Columns::single(4).modify().with(st_green));

    if !wide {
        table.with(Disable::Column(4..))
    } else {
        table
    }
    .with(theme)
}
