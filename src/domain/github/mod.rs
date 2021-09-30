mod asset;
mod release;
mod response;

use std::cmp;

use self::release::Release;
use self::response::GithubResponse;
use super::package::{Package, PackageMatchKind};
use super::util;
use crate::AppError;
use anyhow::Context;
// use console::style;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
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
                let req_url = format!(
                    "https://api.github.com/repos{}/releases/latest",
                    &pkg.repo.path()
                );
                // let req_url = format!("https://api.github.com/repos/{}/releases/latest", &pkg.repo);
                self.get_exact_release(&req_url).await
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos{}/releases/tags/{}",
                    &pkg.repo.path(),
                    &pkg.requested,
                );
                self.get_exact_release(&req_url).await
            }
            PackageMatchKind::SemVer => self.find_release(pkg).await,
        };
        match resp {
            Ok(mut release) => {
                // dbg!(&release);

                // Under normal circumstances, i.e, when not forcing a re-install,
                // or not ensuring existance, if tag of the release is the same,
                // say "nightly", we want to compare its `published_at` date to
                // what we have on record. If it's not the same as ours, skip it.
                // NB: Strict comparison for equality should be faster and enough.
                if !force
                    && pkg.tag.is_some() // newly requested package will have `None`
                    && pkg.timestamp.is_some() // newly requested package will have `None`
                    && &release.tag_name == pkg.tag.as_ref().unwrap()
                    && release.published_at == pkg.timestamp.unwrap()
                {
                    return Ok(false);
                }

                pkg.timestamp = Some(release.published_at);
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

        // dbg!(&resp);
        // dbg!(&resp.status());
        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound);
        }

        let resp = resp
            .json::<GithubResponse<Release>>()
            .await
            .context("parsing latest release response body")?;

        if let GithubResponse::Ok(mut release) = resp {
            release.assets.retain(|asset| {
                util::matches_target(&asset.name)
                    && util::archive_kind(&asset.name) != util::ArchiveKind::Unsupported
            });
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
            "https://api.github.com/repos{}/releases?per_page={}",
            &pkg.repo.path(),
            self.per_page,
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
                    release.assets.retain(|asset| {
                        util::matches_target(&asset.name)
                            && util::archive_kind(&asset.name) != util::ArchiveKind::Unsupported
                    });
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

    // pub async fn download( &self, pb: &ProgressBar, pkg: &mut Package, temp_dir: &TempDir,) -> Result<(), AppError> {
    pub async fn download( &self, pkg: &mut Package, temp_dir: &TempDir,) -> Result<(), AppError> {
        use anyhow::anyhow;
        use reqwest::StatusCode;
        let req_url = format!(
            "https://api.github.com/repos{}/releases/assets/{}",
            &pkg.repo.path(),
            pkg.asset_id.as_ref().unwrap()
        );

        let resp = self
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
        let tot_size = resp.content_length().context("getting content length")?;

        let pb = ProgressBar::new(tot_size);
        pb.set_style(
            ProgressStyle::default_bar()
                // .template("{spinner:.green} {msg}\n[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                // .progress_chars("#>-")
                .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                // .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .progress_chars("##-")
        );
        // let repo_name = util::repo_name(&pkg.repo);
        // let msg = format!("downloading: {}", style(&repo_name).green());
        // pb.set_message(msg);
        // pb.set_message("Downloading");

        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();

        let temp_file_name = temp_dir.path().join(pkg.asset_name.as_ref().unwrap());
        let mut temp_file = File::create(temp_file_name.as_path())
            .await
            .context(format!(
                "creating a temp file: {:?}",
                temp_file_name.as_path(),
            ))?;

        while let Some(item) = stream.next().await {
            let chunk = item.context("retrieving a next chunk")?;
            temp_file
                .write(&chunk)
                .await
                .context("writing a chunk to temp file")?;
            let new = cmp::min(downloaded + (chunk.len() as u64), tot_size);
            downloaded = new;
            pb.set_position(new);
        }

        // pb.finish_with_message(msg);
        pb.finish_and_clear();
        // dbg!(tot_size);
        // dbg!(downloaded);
        // pb.set_position(tot_size);

        pkg.asset_path = Some(temp_file_name);
        Ok(())
    }
}
