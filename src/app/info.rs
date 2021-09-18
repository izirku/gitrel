use anyhow::Result;
use clap::ArgMatches;

use crate::business::conf::{ConfigurationManager, RequestedSpec};
use crate::business::github::GitHub;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let req = RequestedSpec::from_str(repo);
    // let (repo, tag) = parse_repo_spec(repo);
    let gh = GitHub::new(&cm)?;

    let release = gh.find_match(&req).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
