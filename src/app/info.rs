use crate::business::conf::ConfigurationManager;
use crate::business::github::{parse_repo_spec, GitHub};
use anyhow::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap();
    let (repo, tag) = parse_repo_spec(repo);
    let gh = GitHub::new(&cm)?;

    let release = gh.find_match(&repo, &tag).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
