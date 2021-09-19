use crate::business::conf::{ConfigurationManager, Package};
use crate::business::github::{GitHub, GithubResponse};
use anyhow::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let req_pkgs = cm.requested_packages()?;

    let gh = GitHub::new(&cm)?;

    if !matches.is_present("all") {
        unimplemented!();
    }

    for (name, requested) in req_pkgs.into_iter() {
        let pkg = Package {
            name: &name,
            requested: Some(&requested),
            installed: None,
        };
        let release = gh.find_matching_release(&pkg).await?;
        if let GithubResponse::Ok(release) = &release {
            println!("{} {} -> {}", name, &requested.version, &release.tag_name);
        }
    }

    Ok(())
}
