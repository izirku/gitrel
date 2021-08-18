use crate::business::{client, data::github};
use anyhow::Result;

pub async fn info(repo: &str, token: Option<&String>) -> Result<()> {
    let client = client::create(token)?;
    // let base_url = format!("https://api.github.com/repos/{}/", repo);
    //     let latest_release_url = Url::parse(&base_url)?.join("releases/latest")?;
    //
    //     let latest_release = client
    //         .get(latest_release_url)
    //         .send()
    //         .await?
    //         .json::<github::model::Release>()
    //         .await?;
    //     println!("=== LATEST RELEASE ===");
    //     println!("{:#?}", &latest_release);

    let mut page = 1;
    let per_page = 20;
    let max_pages = 3;
    let release = 'outer: loop {
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
            if release.tag_name == "v0.10.0" {
                break 'outer Some(release);
            }
        }

        page += 1;
        if page > max_pages {
            break None;
        }
    };
    println!("found:\n\n{:#?}", &release);
    Ok(())
}

// println!("url: {}", &releases_url);
//     let mut releases_url = Url::parse( &base_url)?.join("releases")?;
//     releases_url.set_query(Some(&per_page));
//     releases_url.query_pairs_mut().append_pair("page", page.to_string());
//     println!("url: {}", releases_url.as_str());
// let releases = client
//     .get(format!("https://api.github.com/repos/{}/releases", repo))
//     .send()
//     .await?
//     .json::<Vec<github::model::Release>>()
//     .await?;
// println!("=== RELEASES ===");
// println!("{:#?}", &releases[..2]);
// println!("releases #: {}", releases.len());
