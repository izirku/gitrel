use crate::business::conf::{ConfigurationManager, Package};
use crate::business::github::{GitHub, GithubResponse};
use crate::Result;
use crate::error::AppError;
use clap::ArgMatches;

pub async fn process(cm: &ConfigurationManager, matches: &ArgMatches) -> Result<()> {
    let req_pkgs = cm.requested_packages()?;
    match cm.installed_packages() {
        Err(e) if e => return 
    }

    let gh = GitHub::new(cm)?;

    if !matches.is_present("all") {
        unimplemented!();
    }

    for (name, requested) in req_pkgs.iter() {
        let pkg = Package::create(name, Some(requested), None);
        let release = gh.get_matching_release(&pkg).await?;

        if let GithubResponse::Ok(release) = &release {
            println!("{} {} -> {}", name, requested.version, release.tag_name);
        }
    }

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
