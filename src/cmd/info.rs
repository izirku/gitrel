use crate::domain::conf::ConfigurationManager;
use crate::domain::github::GitHub;
use crate::domain::package::Package;
use crate::Result;
use clap::ArgMatches;

pub async fn info(matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);

    let cm = ConfigurationManager::with_clap_matches(matches)?;

    // we only want a single client, later will be used by GitLab APIs as well
    let client = reqwest::Client::new();

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    if gh.find_match(&mut pkg).await? {
        println!("found:\n\n{:#?}", &pkg);
    }

    Ok(())
}
