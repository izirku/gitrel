use crate::business::conf::{ConfigurationManager, Package};
use crate::business::github::{GitHub, GithubResponse};
use crate::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);
    let gh = GitHub::new(cm)?;

    let resp = gh.get_matching_release(&pkg).await?;
    if let GithubResponse::Ok(release) = resp {
        pkg.published_at = Some(release.published_at);
        pkg.tag = Some(release.tag_name);
        println!("found:\n\n{:#?}", &pkg);
    }

    Ok(())
}
