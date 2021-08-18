use crate::business::data::conf::model::PackageReqMap;
use crate::business::{client, data::github};
use anyhow::{Context, Result};
use regex::Regex;
use semver::{Version, VersionReq};
use std::fs;
use std::path::Path;

/// List requested packages in a given TOML `file` file.
pub async fn update_requested(file: &Path, token: Option<&String>) -> Result<()> {
    let file = fs::read_to_string(file)
        .with_context(|| format!("unable to read packages file: {:?}", file))?;

    let toml = toml::from_str::<PackageReqMap>(&file)
        .with_context(|| format!("malformed packages TOML file: {:?}", file))?;

    let client = client::create(token)?;
    let semver = Regex::new(r"\d+\.\d+\.\d+(-[-.[:alnum:]]*)?")?;

    // let mut cols = Vec::with_capacity(toml.len());

    for (name, pkg_spec) in toml.into_iter() {
        let ver = pkg_spec.get_version();
        let repo = pkg_spec.get_repo(&name);
        // let repo = format!("[https://github.com/{}]", pkg_spec.get_repo(&name));
        // cols.push(vec![name, pkg_spec.get_version(), repo]);

        let ver_req = VersionReq::parse(&ver)?;
        let mut page = 1;
        let per_page = 20;
        let max_pages = 3;
        let release = 'outer: loop {
            dbg!(page);
            let releases_url = format!(
                "https://api.github.com/repos/{}/releases?per_page={}&page={}",
                repo, per_page, page
            );

            let releases = client
                .get(&releases_url)
                .send()
                .await?
                .json::<Vec<github::model::Release>>()
                .await?;

            for release in releases {
                dbg!(&release.tag_name);
                if let Some(semver) = semver.find(&release.tag_name) {
                    let ver_remote = Version::parse(semver.as_str())?;
                    if ver_req.matches(&ver_remote) {
                        break 'outer Some((release, ver_remote));
                    }
                }
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

        if let Some((_release, ver_remote)) = release {
            println!("{} {} -> {}", name, ver_req, ver_remote);
        }
    }
    Ok(())
}
