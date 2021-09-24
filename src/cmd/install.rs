use crate::domain::conf::ConfigurationManager;
use crate::domain::github::GitHub;
use crate::domain::package::{Package, PackageMap};
use crate::error::AppError;
use crate::Result;
use clap::ArgMatches;

pub async fn install(matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(packages) if packages.contains_key(pkg.name.as_ref().unwrap()) => {
            println!(
                "{} it already installed, use 'update' command to update it",
                &pkg.name.unwrap()
            );
            return Ok(());
        }
        Ok(packages) => packages,
        Err(AppError::NotFound) => PackageMap::new(),
        Err(e) => return Err(e),
    };

    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    if gh.find_match(&mut pkg).await? {
        println!("installing package:\n\n{:#?}", &pkg);

        gh.download(&mut pkg, &temp_dir).await?;
        // let installer = Installer::new(cm)?;
        // installer
        //     .download(&pkg.repo, &pkg.asset_name.unwrap())
        //     .await?;
        let key = pkg.name.as_ref().unwrap().to_owned();
        packages.insert(key, pkg);
        cm.put_packages(&packages)?;
    }
    Ok(())
}
