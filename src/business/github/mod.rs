mod matcher;
mod model;

use crate::business::{conf::ConfigurationManager, rx};
use anyhow::{Context, Result};
use reqwest::{header, Client};

use self::model::Release;

use super::conf::RequestedSpec;

// use super::conf::model::PackageRequested;
//
// pub fn find_requested(name: &str, spec: &PackageRequested) {}
pub struct GitHub {
    client: Client,
    per_page: u32,
    // curr_page: u32,
    max_pages: u32,
}

// struct GitHubBuilder {
//     client: Option<Client>,
//     per_page: Option<u32>,
//     page: Option<u32>,
// }

impl GitHub {
    //     pub fn build(token: Option<&String>) -> Result<GitHub> {
    //         Ok(Self {
    //             client: &client::create(token)?,

    //         })
    //     }
    pub fn new(cm: &ConfigurationManager) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("reqwest"),
        );
        if let Some(token) = &cm.token {
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&token).unwrap(),
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .with_context(|| "creating REST API client has failed.")?;

        Ok(Self {
            client,
            per_page: 20,
            max_pages: 3,
        })
    }

    pub fn per_page(&mut self, per_page: u32) -> &mut Self {
        self.per_page = per_page;
        self
    }

    pub async fn find_match(&self, requested: &RequestedSpec) -> Result<Option<Release>> {
        dbg!(requested);
        Ok(None)
        // let client = client::create(&cm.token)?;
        // let repo = matches.value_of("repo").unwrap();

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

        // dbg!(requested);
        // Ok(None)
        // let mut page = 1;

        // Ok('outer: loop {
        //     dbg!(page);
        //     let releases_url = format!(
        //         "https://api.github.com/repos/{}/releases?per_page={}&page={}",
        //         repo, self.per_page, page
        //     );

        //     let releases = self
        //         .client
        //         .get(&releases_url)
        //         .send()
        //         .await?
        //         .json::<Vec<github::model::Release>>()
        //         .await?;

        //     for release in releases {
        //         if release.tag_name == "v0.10.0" {
        //             break 'outer Some(release);
        //         }
        //     }

        //     page += 1;
        //     if page > self.max_pages {
        //         break None;
        //     }
        // })
    }
}
