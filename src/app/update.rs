use crate::business::data::conf::model::PackageReqMap;
use crate::business::{client, data::github};
// use crate::foundation::consts;
use anyhow::{Context, Result};
// use chrono::NaiveDate;
// use regex::Regex;
// use semver::{Version, VersionReq};
use std::fs;
use std::path::Path;

// enum VersionMatch {
//     SemVer(VersionReq),
//     Named(String),
//     Date((String, NaiveDate)),
//     RegEx(Regex),
// }

/// List requested packages in a given TOML `file` file.
pub async fn update_requested(file: &Path, token: Option<&String>) -> Result<()> {
    let file = fs::read_to_string(file)
        .with_context(|| format!("unable to read packages file: {:?}", file))?;

    let toml = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", file))?;

    let client = client::create(token)?;

    // let mut cols = Vec::with_capacity(toml.len());

    for (name, pkg_spec) in toml.into_iter() {
        let pkg_spec = pkg_spec.into_detailed(&name);
        // let ver = pkg_spec.get_version();
        // let repo = pkg_spec.get_repo(&name);
        // let repo = format!("[https://github.com/{}]", pkg_spec.get_repo(&name));
        // cols.push(vec![name, pkg_spec.get_version(), repo]);

        // let ver_req = match pkg_spec.match_kind {};
        // let ver_req = VersionReq::parse(&pkg_spec.matches)?;
        let mut page = 1;
        let per_page = 20;
        let max_pages = 1;
        let release = 'outer: loop {
            dbg!(page);
            let releases_url = format!(
                "https://api.github.com/repos/{}/releases?per_page={}&page={}",
                pkg_spec.repo.as_ref().unwrap(),
                per_page,
                page
            );

            let releases = client
                .get(&releases_url)
                .send()
                .await?
                .json::<Vec<github::model::Release>>()
                .await?;

            for release in releases {
                if pkg_spec.matches(&release.tag_name)? {
                    break 'outer Some(release);
                }
                // if let Some(name) = &release.name {
                //     // dbg!(&release);
                //     // dbg!(name);
                //     if let Some(semver) = consts::SEMVER.find(name) {
                //         let ver_remote = Version::parse(semver.as_str())?;
                //         if ver_req.matches(&ver_remote) {
                //             break 'outer Some((release, ver_remote));
                //         }
                //     }
                // }
                // if release.tag_name == "v0.10.0" {
                //     break 'outer Some(release);
                // }
            }

            page += 1;
            if page > max_pages {
                break None;
            }
        };

        // println!("found:\n\n{:#?}", &release);

        if let Some(release) = &release {
            println!("{} {} -> {}", name, &pkg_spec.matches, &release.tag_name);
        }
    }
    Ok(())
}
