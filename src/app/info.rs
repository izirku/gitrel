use crate::business::conf::{ConfigurationManager, Package, RequestedPackage};
use crate::business::github::GitHub;
use anyhow::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let name = if repo.contains("/") {
        repo.split_at(repo.find('/').unwrap())
            .1
            .get(1..)
            .unwrap()
            .to_lowercase()
    } else {
        repo.to_lowercase()
    };
    let requested = RequestedPackage::create(repo, cm.strip);
    let pkg = Package::create(&name, Some(&requested), None);
    let gh = GitHub::new(&cm)?;

    let release = gh.get_matching_release(&pkg).await?;
    println!("found:\n\n{:#?}", &release);

    Ok(())
}
