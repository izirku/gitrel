use crate::business::conf::{ConfigurationManager, Package, PackageMap};
use crate::business::github::{GitHub, GithubResponse};
use crate::error::AppError;
use crate::Result;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);

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

    let gh = GitHub::new(cm)?;
    let resp = gh.get_matching_release(&pkg).await?;
    if let GithubResponse::Ok(release) = resp {
        pkg.published_at = Some(release.published_at);
        pkg.tag = Some(release.tag_name);

        println!("installing package:\n\n{:#?}", &pkg);

        let key = pkg.name.as_ref().unwrap().to_owned();
        packages.insert(key, pkg);
        cm.put_packages(&packages)?;
    }
    Ok(())
}
