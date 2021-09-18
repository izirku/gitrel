use anyhow::Result;
use clap::ArgMatches;

use crate::business::conf::{ConfigurationManager, Package, RequestedPackage};
use crate::business::github::GitHub;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let requested = RequestedPackage::from_str(repo);
    let bin = if repo.contains("/") {
        repo.split_at(repo.find('/').unwrap()).1.get(1..).unwrap()
    } else {
        repo
    };
    let pkg = Package {
        bin,
        requested: Some(&requested),
        installed: None,
    };
    // let (repo, tag) = parse_repo_spec(repo);
    let gh = GitHub::new(&cm)?;

    let release = gh.find_match(&pkg).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
