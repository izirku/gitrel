use crate::business::conf::{
    ConfigurationManager, InstalledPackage, InstalledPackageMap, Package, RequestedPackage,
};
use crate::business::github::{GitHub, GithubResponse};
use crate::error::AppError;
use crate::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let name = if repo.contains('/') {
        repo.split_at(repo.find('/').unwrap())
            .1
            .get(1..)
            .unwrap()
            .to_lowercase()
    } else {
        repo.to_lowercase()
    };

    let mut installed = match cm.get_installed_packages() {
        Ok(installed_pkgs) if installed_pkgs.contains_key(&name) => {
            println!(
                "{} it already installed, use 'update' command to update it",
                &name
            );
            return Ok(());
        }
        Ok(installed_pkgs) => installed_pkgs,
        Err(AppError::NotFound) => InstalledPackageMap::new(),
        Err(e) => return Err(e),
    };

    let requested = RequestedPackage::create(repo, cm.strip);
    let pkg = Package::create(&name, Some(&requested), None);
    let gh = GitHub::new(cm)?;
    let resp = gh.get_matching_release(&pkg).await?;
    if let GithubResponse::Ok(release) = resp {
        println!("found:\n\n{:#?}", &release);
        let installed_pkg =
            InstalledPackage::create(repo, &release.tag_name, &release.published_at);
        installed.insert(name, installed_pkg);
        cm.put_installed_packages(&installed)?;
    }
    Ok(())
}
