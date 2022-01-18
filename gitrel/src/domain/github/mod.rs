mod asset;
mod release;
mod response;

use self::release::Release;
use self::response::GithubResponse;
use super::package::{match_kind, Package, PackageMatchKind};
use super::util;
use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client, Method};
use std::cmp;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

const GH_MAX_PAGES: usize = 5;
const GH_PER_PAGE: usize = 25;

pub struct GitHub {
    client: Client,
    api_headers: header::HeaderMap,
    dl_headers: header::HeaderMap,
}

impl GitHub {
    pub fn create(token: Option<&String>) -> Self {
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
            let token = format!("token {}", token);

            api_headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&token).unwrap(),
            );
            dl_headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&token).unwrap(),
            );
        }
        // dbg!(&api_headers);

        // let client = ;

        Self {
            client: reqwest::Client::new(),
            api_headers,
            dl_headers,
        }
    }

    // pub fn per_page(&mut self, per_page: u32) -> &mut Self {
    //     self.per_page = per_page;
    //     self
    // }

    /// Find a `Release` matching provided parameters.
    pub async fn find_new(
        &self,
        user: &str,
        repo: &str,
        requested: &str,
    ) -> Result<Option<Release>> {
        match match_kind(requested) {
            PackageMatchKind::Latest => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    user, repo,
                );
                self.get_exact_release(&req_url, repo).await
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/tags/{}",
                    user, repo, requested,
                );
                self.get_exact_release(&req_url, repo).await
            }
            PackageMatchKind::SemVer => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases?per_page={}",
                    user, repo, GH_PER_PAGE,
                );
                self.find_release(&req_url, requested, repo).await
            }
        }
    }

    /// Find a `Release` matching provided `Package`.
    /// When `force` is `true`, return `Release`, even if it's not newer than
    /// the one specified in `Package`
    pub async fn find_update(&self, package: &Package, force: bool) -> Result<Option<Release>> {
        let res = self
            .find_new(&package.user, &package.repo, &package.requested)
            .await?;

        match res {
            Some(release) => {
                // Under normal circumstances, i.e, when not forcing a re-install,
                // or not ensuring existance, if tag of the release is the same,
                // say "nightly", we want to compare its `published_at` date to
                // what we have on record. If it's the same as ours, skip it.
                // NB: Strict comparison for equality should be faster and enough.
                if !force
                    && release.tag_name == package.tag
                    && release.published_at == package.timestamp
                {
                    Ok(None)
                } else {
                    Ok(Some(release))
                }
            }
            None => Ok(None),
        }
    }

    // TODO:add FilterKind = Contains | RegEx
    async fn get_exact_release(&self, req_url: &str, filter: &str) -> Result<Option<Release>> {
        let resp = self
            .client
            .get(req_url)
            .headers(self.api_headers.clone())
            .send()
            .await
            .context("fetching latest release")?;

        // dbg!(&resp);
        // dbg!(&resp.status());
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }

        let resp = resp
            .json::<GithubResponse<Release>>()
            .await
            .context("parsing latest release response body")?;

        // dbg!(&resp);

        match resp {
            GithubResponse::Ok(mut release) => {
                release.assets.retain(|asset| {
                    util::matches_target(&asset.name)
                        && util::archive_kind(&asset.name) != util::ArchiveKind::Unsupported
                        && asset.name.contains(filter)
                });
                if !release.assets.is_empty() {
                    Ok(Some(release))
                } else {
                    Ok(None)
                }
                // match release.assets.len() {
                //     1 => Ok(Some(release)),
                //     0 => Ok(None),
                //     _ => {
                //         // dbg!(release.assets);
                //         Err(anyhow!("multiple assets matched the filter, consider filing a bug report stating which repo it failed on"))
                //     }
                // }
            }
            GithubResponse::Err(err) => {
                eprintln!("{}", err.message);
                Ok(None)
            }
        }
    }

    async fn find_release(
        &self,
        req_url: &str,
        requested: &str,
        filter: &str,
    ) -> Result<Option<Release>> {
        let mut curr_page: usize = 1;

        'outer: loop {
            // dbg!(curr_page);

            let resp = self
                .client
                .request(Method::GET, req_url)
                .headers(self.api_headers.clone())
                .query(&[("page", curr_page)])
                .send()
                .await
                .context("fething next page")?;

            // dbg!(resp.status());

            if resp.status().as_u16() != 200 {
                return Ok(None);
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
                if util::matches_semver(&release.tag_name, requested) {
                    release.assets.retain(|asset| {
                        util::matches_target(&asset.name)
                            && util::archive_kind(&asset.name) != util::ArchiveKind::Unsupported
                            && asset.name.contains(filter)
                    });

                    if !release.assets.is_empty() {
                        break 'outer Ok(Some(release));
                    } else {
                        break 'outer Ok(None);
                    }
                }
            }

            curr_page += 1;
            if curr_page > GH_MAX_PAGES {
                // break Err(AppError::NotFound);
                break Ok(None);
            }
        }
    }

    // pub async fn download( &self, pb: &ProgressBar, pkg: &mut Package, temp_dir: &TempDir,) -> Result<(), AppError> {
    pub async fn download(
        &self,
        user: &str,
        repo: &str,
        asset_id: u64,
        asset_name: &str,
        temp_dir: &TempDir,
    ) -> Result<PathBuf> {
        use reqwest::StatusCode;
        let req_url = format!(
            "https://api.github.com/repos/{}/{}/releases/assets/{}",
            user, repo, asset_id
        );

        let resp = self
            .client
            .get(&req_url)
            .headers(self.dl_headers.clone())
            .send()
            .await
            .context("fething an asset")?;

        if resp.status() != StatusCode::OK {
            let mut msg = format!("getting: {}", &req_url);
            if let Ok(txt) = resp.text().await {
                msg.push('\n');
                msg.push_str(&txt);
            }
            return Err(anyhow!(msg));
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

        let temp_file_name = temp_dir.path().join(asset_name);
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

        Ok(temp_file_name)
    }
}
