use crate::business::data::conf::ConfigurationManager;
use crate::business::data::github::GitHub;
use anyhow::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap();
    let gh = GitHub::new(&cm)?;

    let release = gh.find_match(repo).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
