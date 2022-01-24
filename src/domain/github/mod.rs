mod asset;
mod release;
mod response;

use std::cmp;
use std::fmt::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client, Method};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::domain::github::response::ErrorResponse;

use self::release::Release;
use self::response::GithubResponse;
use super::error::GithubError;
use super::package::{match_kind, Package, PackageMatchKind};
use super::util;

type Result<T, E = GithubError> = std::result::Result<T, E>;

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

        Self {
            client: reqwest::Client::new(),
            api_headers,
            dl_headers,
        }
    }

    /// Find a `Release` matching provided parameters.
    pub async fn find_new(
        &self,
        user: &str,
        repo: &str,
        requested: &str,
        asset_glob: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Release> {
        match match_kind(requested) {
            PackageMatchKind::Latest => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    user, repo,
                );
                self.find_exact_release(&req_url, repo, asset_glob, asset_re)
                    .await
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/tags/{}",
                    user, repo, requested,
                );
                self.find_exact_release(&req_url, repo, asset_glob, asset_re)
                    .await
            }
            PackageMatchKind::SemVer => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases?per_page={}",
                    user, repo, GH_PER_PAGE,
                );
                self.find_release(&req_url, requested, repo, asset_glob, asset_re)
                    .await
            }
        }
    }

    /// Find a `Release` matching provided `Package`.
    /// When `force` is `true`, return `Release`, even if it's not newer than
    /// the one specified in `Package`
    pub async fn find_existing(&self, package: &Package) -> Result<Release> {
        let res = self
            .find_new(
                &package.user,
                &package.repo,
                &package.requested,
                package.asset_glob.as_deref(),
                package.asset_re.as_deref(),
            )
            .await;

        if let Ok(release) = res {
            // we want to compare release's `published_at` date to
            // what we have on record. If it's the same as ours, skip it.
            // NB: Strict comparison for equality should be faster and enough.
            if release.tag_name == package.tag && release.published_at == package.timestamp {
                Err(GithubError::AlreadyUpToDate)
            } else {
                Ok(release)
            }
        } else {
            res
        }
    }

    async fn find_exact_release(
        &self,
        req_url: &str,
        repo: &str,
        asset_glob: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Release> {
        use reqwest::StatusCode;

        let resp = self
            .client
            .get(req_url)
            .headers(self.api_headers.clone())
            .send()
            .await
            .context("fetching latest release")?;

        if resp.status() == StatusCode::NOT_FOUND {
            return Err(GithubError::ReleaseNotFound);
        }

        if resp.status() != StatusCode::OK {
            return Err(GithubError::AnyHow(anyhow!("getting")));
        }

        let resp: GithubResponse<Release> = resp
            .json()
            .await
            .context("parsing latest release response body")?;

        match resp {
            GithubResponse::Ok(mut release) => {
                let asset_matcher = get_asset_name_matcher(repo, asset_glob, asset_re)?;
                release.assets.retain(|asset| asset_matcher(&asset.name));

                match release.assets.len() {
                    1 => Ok(release),
                    0 => Err(GithubError::AssetNoMatch),
                    _ => {
                        let mut msg: String = String::new();
                        for asset in &release.assets {
                            writeln!(
                                msg,
                                "  {} ({})",
                                &asset.name,
                                bytesize::to_string(asset.size, false)
                            )
                            .map_err(anyhow::Error::msg)?;
                        }
                        Err(GithubError::AssetMultipleMatch(msg))
                    }
                }
            }
            GithubResponse::Err(ErrorResponse { message }) => {
                Err(GithubError::AnyHow(anyhow!(message)))
            }
        }
    }

    async fn find_release(
        &self,
        req_url: &str,
        requested: &str,
        repo: &str,
        asset_glob: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Release> {
        use reqwest::StatusCode;
        let asset_matcher = get_asset_name_matcher(repo, asset_glob, asset_re)?;
        let mut curr_page: usize = 1;

        'outer: loop {
            let resp = self
                .client
                .request(Method::GET, req_url)
                .headers(self.api_headers.clone())
                .query(&[("page", curr_page)])
                .send()
                .await
                .context("sending request")?;

            if resp.status() == StatusCode::NOT_FOUND {
                return Err(GithubError::ReleaseNotFound);
            }

            if resp.status() != StatusCode::OK {
                return Err(GithubError::AnyHow(anyhow!("getting")));
            }

            let releases: GithubResponse<Vec<Release>> =
                resp.json().await.context("parsing response body")?;

            let releases = match releases {
                GithubResponse::Ok(res) => res,
                GithubResponse::Err(ErrorResponse { message }) => {
                    return Err(GithubError::AnyHow(anyhow!(message)));
                }
            };

            for mut release in releases {
                if util::matches_semver(&release.tag_name, requested) {
                    release.assets.retain(|asset| asset_matcher(&asset.name));

                    match release.assets.len() {
                        1 => break 'outer Ok(release),
                        0 => break 'outer Err(GithubError::ReleaseNotFound),
                        _ => {
                            let mut msg: String = String::new();
                            for asset in &release.assets {
                                writeln!(
                                    msg,
                                    "  {} ({})",
                                    &asset.name,
                                    bytesize::to_string(asset.size, false)
                                )
                                .map_err(anyhow::Error::msg)?;
                            }
                            break 'outer Err(GithubError::AssetMultipleMatch(msg));
                        }
                    }
                }
            }

            curr_page += 1;
            if curr_page > GH_MAX_PAGES {
                break Err(GithubError::ReleaseNotFound);
            }
        }
    }

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

        if resp.status() == StatusCode::NOT_FOUND {
            return Err(GithubError::AssetNotFound);
        }

        if resp.status() != StatusCode::OK {
            let mut msg = format!("getting: {}", &req_url);
            if let Ok(txt) = resp.text().await {
                msg.push('\n');
                msg.push_str(&txt);
            }
            return Err(GithubError::AnyHow(anyhow!(msg)));
        }
        let tot_size = resp.content_length().context("getting content length")?;

        let pb = ProgressBar::new(tot_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .progress_chars("##-")
        );

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

        pb.finish_and_clear();

        Ok(temp_file_name)
    }
}

fn get_asset_name_matcher(
    repo: &str,
    asset_glob: Option<&str>,
    asset_re: Option<&str>,
) -> Result<Box<dyn Fn(&str) -> bool>> {
    if let Some(s) = asset_glob {
        if s.contains('/') || s.contains("**") {
            return Err(GithubError::AnyHow(anyhow!("'/' or '**' are not allowed not allowed in a glob pattern matching a single file name")));
        }
        let glob = glob::Pattern::new(s).context("invalid asset name glob pattern")?;
        Ok(Box::new(move |asset_name: &str| glob.matches(asset_name)))
    } else if let Some(s) = asset_re {
        let re = regex::Regex::new(s).context("invalid asset name RegEx pattern")?;
        Ok(Box::new(move |asset_name: &str| re.is_match(asset_name)))
    } else {
        let repo = repo.to_owned();
        Ok(Box::new(move |asset_name: &str| {
            util::matches_target(asset_name) && asset_name.contains(&repo)
        }))
    }
}
