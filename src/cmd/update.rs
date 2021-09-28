use crate::domain::{installer, util};
use crate::domain::util::parse_gh_repo_spec;
use crate::domain::{conf::ConfigurationManager, github::GitHub};
use crate::{AppError, Result};
use anyhow::Context;
use clap::{crate_name, ArgMatches};

/// Update installed packages
pub async fn update(matches: &ArgMatches) -> Result<()> {
    let cm = ConfigurationManager::with_clap_matches(matches)?;

    let mut packages = match cm.get_packages() {
        Ok(packages) => packages,
        Err(AppError::NotFound) => {
            println!(
                "No managed installationts on this system. Use `{} install  repo@[*|name|semver]` to install a package",
                crate_name!(),
            );
            return Ok(());
        }
        Err(e) => return Err(e),
    };

    let mut needs_save = false;
    let client = reqwest::Client::new();
    let temp_dir = tempfile::tempdir().expect("creating a temp dir failed");

    let gh = GitHub::create(&client, cm.token.as_ref(), cm.gh_per_page, cm.gh_max_pages);

    // update --all packages
    if matches.is_present("all") {
        for (name, pkg) in &mut packages {
            // pkg.name = Some(name.to_owned());
            if gh.find_match(pkg, false).await? {
                println!("updating package: {}", &name);

                gh.download(pkg, &temp_dir).await?;
                installer::install(pkg, &cm.bin_dir, cm.strip).await?;
                needs_save = true;
                // let key = pkg.name.as_ref().unwrap().to_owned();
                // packages.insert(key, pkg);
            }
        }
    }

    // update a single package
    if let Some(repo_spec) = matches.value_of("repo") {
        // let (_repo, repo_name, requested) = parse_gh_repo_spec(repo);
        let (repo_url, requested) = parse_gh_repo_spec(repo_spec);
        let repo_name = util::repo_name(&repo_url);
        if !packages.contains_key(&repo_name) {
            println!(
                "{1} it not installed on this system. Use `{0} install  {1}` to install a package",
                crate_name!(),
                repo_spec,
            );
            return Ok(());
        }

        let pkg = packages
            .get_mut(&repo_name)
            .context("failed to read a package spec from installed packages registry")?;

        pkg.requested = requested;

        if gh.find_match(pkg, false).await? {
            gh.download(pkg, &temp_dir).await?;
            installer::install(pkg, &cm.bin_dir, cm.strip).await?;
            needs_save = true;
        }
    }

    if needs_save {
        cm.put_packages(&packages)?;
    }
    //     let req_pkgs = cm.requested_packages()?;
    //     match cm.installed_packages() {
    //         Err(e) if e => return,
    //     }

    //     let gh = GitHub::new(cm)?;

    //     if !matches.is_present("all") {
    //         unimplemented!();
    //     }

    //     for (name, requested) in req_pkgs.iter() {
    //         let pkg = Package::create(name, Some(requested), None);
    //         let release = gh.get_matching_release(&pkg).await?;

    //         if let GithubResponse::Ok(release) = &release {
    //             println!("{} {} -> {}", name, requested.version, release.tag_name);
    //         }
    //     }

    Ok(())
}
// use anyhow::{Context, Result};
// use std::fs;

// use crate::business::{
//     conf::{requested::PackageReqMap, ConfigurationManager},
//     github::GitHub,
// };

// // use chrono::NaiveDate;
// // use regex::Regex;
// // use semver::{Version, VersionReq};
// // use crate::foundation::consts;

// // enum VersionMatch {
// //     SemVer(VersionReq),
// //     Named(String),
// //     Date((String, NaiveDate)),
// //     RegEx(Regex),
// // }

// /// List requested packages in a given TOML `file` file.
// pub async fn update_requested(cm: &ConfigurationManager) -> Result<()> {
//     let file = fs::read_to_string(cm.requested.as_path())
//         .with_context(|| format!("unable to read packages file: {:?}", cm.requested))?;

//     let toml = toml::from_str::<PackageReqMap>(&file)
//         .with_context(|| format!("malformed packages TOML file: {:?}", cm.requested))?;

//     let gh = GitHub::new(cm)?;

//     // let mut cols = Vec::with_capacity(toml.len());

//     for (name, pkg_spec) in toml.into_iter() {
//         let pkg_spec = pkg_spec.into_detailed(&name);
//         // let ver = pkg_spec.get_version();
//         // let repo = pkg_spec.get_repo(&name);
//         // let repo = format!("[https://github.com/{}]", pkg_spec.get_repo(&name));
//         // cols.push(vec![name, pkg_spec.get_version(), repo]);

//         // let ver_req = match pkg_spec.match_kind {};
//         // let ver_req = VersionReq::parse(&pkg_spec.matches)?;
//         let mut page = 1;
//         let per_page = 20;
//         let max_pages = 1;
//         // let release = gh.find_match(pkg_spec.repo.as_ref().unwrap()).await?;

//         let release = 'outer: loop {
//             dbg!(page);
//             let releases_url = format!(
//                 "https://api.github.com/repos/{}/releases?per_page={}&page={}",
//                 pkg_spec.repo.as_ref().unwrap(),
//                 per_page,
//                 page
//             );

//             let releases = client
//                 .get(&releases_url)
//                 .send()
//                 .await?
//                 .json::<Vec<github::model::Release>>()
//                 .await?;

//             for release in releases {
//                 if pkg_spec.matches(&release.tag_name)? {
//                     break 'outer Some(release);
//                 }
//                 // if let Some(name) = &release.name {
//                 //     // dbg!(&release);
//                 //     // dbg!(name);
//                 //     if let Some(semver) = consts::SEMVER.find(name) {
//                 //         let ver_remote = Version::parse(semver.as_str())?;
//                 //         if ver_req.matches(&ver_remote) {
//                 //             break 'outer Some((release, ver_remote));
//                 //         }
//                 //     }
//                 // }
//                 // if release.tag_name == "v0.10.0" {
//                 //     break 'outer Some(release);
//                 // }
//             }

//             page += 1;
//             if page > max_pages {
//                 break None;
//             }
//         };

//         // println!("found:\n\n{:#?}", &release);

//         if let Some(release) = &release {
//             println!("{} {} -> {}", name, &pkg_spec.matches, &release.tag_name);
//         }
//     }
//     Ok(())
// }
