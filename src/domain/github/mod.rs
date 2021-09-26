mod asset;
mod release;
mod response;

// use self::asset::Asset;
use self::release::Release;
use self::response::GithubResponse;
use super::util;
use super::package::{Package, PackageMatchKind};
use crate::error::AppError;
use anyhow::Context;
use reqwest::{header, Client, Method};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
// use tokio::io::{self, AsyncWriteExt};

pub struct GitHub<'a> {
    client: &'a Client,
    api_headers: header::HeaderMap,
    dl_headers: header::HeaderMap,
    per_page: usize,
    max_pages: usize,
}

impl<'a> GitHub<'a> {
    pub fn create(
        client: &'a Client,
        token: Option<&'a String>,
        per_page: usize,
        max_pages: usize,
    ) -> Self {
        let mut api_headers = header::HeaderMap::new();
        api_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github.v3+json"),
        );
        api_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("reqwest"),
        );

        let mut dl_headers = header::HeaderMap::new();
        dl_headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/octet-stream"),
        );
        dl_headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("reqwest"),
        );

        if let Some(token) = token {
            api_headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(token).unwrap(),
            );
            dl_headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(token).unwrap(),
            );
        }

        Self {
            client,
            api_headers,
            dl_headers,
            per_page,
            max_pages,
        }
    }

    // pub fn per_page(&mut self, per_page: u32) -> &mut Self {
    //     self.per_page = per_page;
    //     self
    // }

    pub async fn find_match(&self, pkg: &mut Package, force: bool) -> Result<bool, AppError> {
        let resp = match pkg.match_kind() {
            PackageMatchKind::Latest => {
                let req_url = format!("https://api.github.com/repos/{}/releases/latest", &pkg.repo);
                self.get_exact_release(&req_url).await
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/releases/tags/{}",
                    &pkg.repo, &pkg.requested,
                );
                self.get_exact_release(&req_url).await
            }
            PackageMatchKind::SemVer => self.find_release(pkg).await,
        };
        match resp {
            Ok(mut release) => {
                if !force && &release.tag_name == pkg.tag.as_ref().unwrap() && release.published_at <= pkg.published_at.unwrap() {
                    return Ok(false);
                }
                pkg.published_at = Some(release.published_at);
                pkg.tag = Some(release.tag_name);
                let asset = release.assets.pop().unwrap();
                pkg.asset_id = Some(asset.id.to_string());
                pkg.asset_name = Some(asset.name);
                Ok(true)
            }
            Err(AppError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn get_exact_release(&self, req_url: &str) -> Result<Release, AppError> {
        let resp = self
            .client
            .get(req_url)
            .headers(self.api_headers.clone())
            .send()
            .await
            .context("fething latest release")?;

        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound);
        }

        let resp = resp
            .json::<GithubResponse<Release>>()
            .await
            .context("parsing latest release response body")?;

        if let GithubResponse::Ok(mut release) = resp {
            release
                .assets
                .retain(|asset| util::matches_target(&asset.name));
            match release.assets.len() {
                1 => Ok(release),
                0 => Err(AppError::NotFound),
                _ => {
                    dbg!(&release);
                    Err(AppError::MultipleResults)
                }
            }
        } else {
            Err(AppError::NotFound)
        }
    }

    async fn find_release(&self, pkg: &Package) -> Result<Release, AppError> {
        let req_url = format!(
            "https://api.github.com/repos/{}/releases?per_page={}",
            &pkg.repo, self.per_page,
        );

        let mut curr_page = 1;

        'outer: loop {
            dbg!(curr_page);

            let resp = self
                .client
                .request(Method::GET, &req_url)
                .headers(self.api_headers.clone())
                .query(&[("page", curr_page)])
                .send()
                .await
                .context("fething next page")?;

            dbg!(resp.status());

            if resp.status().as_u16() != 200 {
                return Err(AppError::NotFound);
            }

            let releases: Vec<GithubResponse<Release>> =
                resp.json().await.context("parsing response body")?;

            for mut release in releases.into_iter().filter_map(|resp| {
                if let GithubResponse::Ok(release) = resp {
                    Some(release)
                } else {
                    None
                }
            }) {
                if util::matches_semver(&release.tag_name, &pkg.requested) {
                    release
                        .assets
                        .retain(|asset| util::matches_target(&asset.name));
                    if release.assets.len() == 1 {
                        break 'outer Ok(release);
                    }
                }
            }

            curr_page += 1;
            if curr_page > self.max_pages {
                break Err(AppError::NotFound);
            }
        }
    }

    pub async fn download(&self, pkg: &mut Package, temp_dir: &TempDir) -> Result<(), AppError> {
        use anyhow::anyhow;
        use reqwest::StatusCode;
        let req_url = format!(
            "https://api.github.com/repos/{}/releases/assets/{}",
            &pkg.repo,
            pkg.asset_id.as_ref().unwrap()
        );

        let mut resp = self
            .client
            .get(&req_url)
            .headers(self.dl_headers.clone())
            .send()
            .await
            .context("fething an asset")?;
        if resp.status() != StatusCode::OK {
            dbg!(&resp);
            let mut msg = format!("getting: {}", &req_url);
            if let Ok(txt) = resp.text().await {
                msg.push('\n');
                msg.push_str(&txt);
            }
            return Err(AppError::AnyHow(anyhow!(msg)));
        }

        let temp_file_name = temp_dir.path().join(pkg.asset_name.as_ref().unwrap());
        let mut temp_file = File::create(temp_file_name.as_path())
            .await
            .context(format!(
                "creating a temp file: {:?}",
                temp_file_name.as_path(),
            ))?;

        while let Some(chunk) = resp.chunk().await.context("retrieving a next chunk")? {
            temp_file
                .write_all(&chunk)
                .await
                .context("writing a chunk to temp file")?;
        }
        println!("temp file created: {:?}", &temp_file_name);

        pkg.asset_path = Some(temp_file_name);
        Ok(())
    }
}
