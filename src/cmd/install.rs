use crate::domain::conf::ConfigurationManager;
use crate::domain::github::GitHub;
use crate::domain::installer;
use crate::domain::package::{Package, PackageMap};
use crate::error::AppError;
use crate::Result;
use clap::{crate_name, ArgMatches};

// Install packages
pub async fn install(matches: &ArgMatches) -> Result<()> {
    let repo = matches.value_of("repo").unwrap(); // required arg, safe to unwrap
    let mut pkg = Package::create(repo);
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => PackageMap::new(),
        Err(e) => return Err(e),
    };

    let force_reinstall = matches.is_present("force");
    if !force_reinstall && packages.contains_key(pkg.name.as_ref().unwrap()) {
        println!(
                "{} it already installed, use `{1} install --force {2}` to reinstall, or `{1} update ...` to update",
                &pkg.name.unwrap(),
                crate_name!(),
                repo,
            );
        return Ok(());
    }

    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    if gh.find_match(&mut pkg, force_reinstall).await? {
        println!("installing package:\n\n{:#?}", &pkg);

        gh.download(&mut pkg, &temp_dir).await?;
        installer::install(&pkg, &cm.bin_dir).await?;
        let key = pkg.name.as_ref().unwrap().to_owned();
        packages.insert(key, pkg);
        cm.put_packages(&packages)?;
    }
    Ok(())
}
