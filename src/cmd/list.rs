use crate::domain::conf::ConfigurationManager;
use crate::{AppError, Result};
use clap::ArgMatches;
use console::style;
use tabled::{style::Line, Alignment, Column, Format, Modify, Object, Row, Style, Table, Tabled};

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
pub fn list(matches: &ArgMatches) -> Result<()> {
    let cm = ConfigurationManager::with_clap_matches(matches)?;
    let packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => {
            println!("nothing is installed on this system");
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut list_lines = Vec::with_capacity(packages.len());

    let blank = "".to_string();
    for (name, pkg_spec) in packages.iter() {
        list_lines.push(ListLine {
            bin: name,
            requested: &pkg_spec.requested,
            installed: pkg_spec.tag.as_ref().unwrap_or(&blank),
            repository: format!("https://github.com/{}", &pkg_spec.repo),
        });
    }

    let table = Table::new(&list_lines)
        .with(Style::noborder().header(Some(Line::short('-', '+'))))
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
            format!(
                "{}{}{}",
                style('[').cyan().to_string(),
                s,
                style(']').cyan().to_string()
            )
        })));

    println!("{}", table);

    Ok(())
}
