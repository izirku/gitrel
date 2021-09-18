use anyhow::Result;
use clap::ArgMatches;

use crate::business::conf::{ConfigurationManager, Package, RequestedPackage};
use crate::business::github::GitHub;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let requested = RequestedPackage::create(repo, cm.strip);
    let name = if repo.contains("/") {
        repo.split_at(repo.find('/').unwrap()).1.get(1..).unwrap().to_lowercase()
    } else {
        repo.to_lowercase()
    };
    let pkg = Package {
        name: &name,
        requested: Some(&requested),
        installed: None,
    };
    // let (repo, tag) = parse_repo_spec(repo);
    let gh = GitHub::new(&cm)?;

    let release = gh.get_latest(&pkg).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
