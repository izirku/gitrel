mod asset;
mod release;
mod response;

use std::cmp;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{header, Client, Method};
use tempfile::TempDir;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use self::release::Release;
use self::response::GithubResponse;
use super::package::{match_kind, Package, PackageMatchKind};
use super::util;

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
        asset_contains: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Option<Release>> {
        match match_kind(requested) {
            PackageMatchKind::Latest => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/latest",
                    user, repo,
                );
                self.find_exact_release(&req_url, repo, asset_contains, asset_re)
                    .await
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/tags/{}",
                    user, repo, requested,
                );
                self.find_exact_release(&req_url, repo, asset_contains, asset_re)
                    .await
            }
            PackageMatchKind::SemVer => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases?per_page={}",
                    user, repo, GH_PER_PAGE,
                );
                self.find_release(&req_url, requested, repo, asset_contains, asset_re)
                    .await
            }
        }
    }

    /// Find a `Release` matching provided `Package`.
    /// When `force` is `true`, return `Release`, even if it's not newer than
    /// the one specified in `Package`
    pub async fn find_existing(&self, package: &Package) -> Result<Option<Release>> {
        let res = self
            .find_new(
                &package.user,
                &package.repo,
                &package.requested,
                package.asset_contains.as_deref(),
                package.asset_re.as_deref(),
            )
            .await?;

        match res {
            Some(release) => {
                // we want to compare release's `published_at` date to
                // what we have on record. If it's the same as ours, skip it.
                // NB: Strict comparison for equality should be faster and enough.
                if release.tag_name == package.tag && release.published_at == package.timestamp {
                    Ok(None)
                } else {
                    Ok(Some(release))
                }
            }
            None => Ok(None),
        }
    }

    async fn find_exact_release(
        &self,
        req_url: &str,
        repo: &str,
        asset_contains: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Option<Release>> {
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
                let name_matcher: Box<dyn Fn(&str) -> bool> = if let Some(s) = asset_contains {
                    Box::new(move |asset_name: &str| asset_name == s)
                } else if let Some(s) = asset_re {
                    let re = regex::Regex::new(s).context("invalid asset name RegEx expression")?;
                    Box::new(move |asset_name: &str| re.is_match(asset_name))
                } else {
                    Box::new(|asset_name: &str| {
                        util::matches_target(asset_name) && asset_name.contains(repo)
                    })
                };

                release.assets.retain(|asset| name_matcher(&asset.name));

                match release.assets.len() {
                    1 => Ok(Some(release)),
                    0 => Ok(None),
                    _ => {
                        // dbg!(release.assets);
                        let msg = if asset_re.is_some() {
                            "multiple assets matched, consider modifying `--asset-regex-match` expression"
                        } else if asset_contains.is_some() {
                            "multiple assets matched, consider modifying `--asset-contains` filter or using a more powerful `--asset-regex-match`"
                        } else {
                            "multiple assets matched, consider using `--asset-contains` or `--asset-re` filter"
                        };

                        Err(anyhow!(msg))
                    }
                }
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
        repo: &str,
        asset_contains: Option<&str>,
        asset_re: Option<&str>,
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
                    let name_matcher: Box<dyn Fn(&str) -> bool> = if let Some(s) = asset_contains {
                        Box::new(move |asset_name: &str| asset_name == s)
                    } else if let Some(s) = asset_re {
                        let re =
                            regex::Regex::new(s).context("invalid asset name RegEx expression")?;
                        Box::new(move |asset_name: &str| re.is_match(asset_name))
                    } else {
                        Box::new(|asset_name: &str| {
                            util::matches_target(asset_name) && asset_name.contains(repo)
                        })
                    };

                    release.assets.retain(|asset| name_matcher(&asset.name));

                    match release.assets.len() {
                        1 => break 'outer Ok(Some(release)),
                        0 => break 'outer Ok(None),
                        _ => {
                            // dbg!(release.assets);
                            let msg = if asset_re.is_some() {
                                "multiple assets matched, consider modifying `--asset-regex-match` expression"
                            } else if asset_contains.is_some() {
                                "multiple assets matched, consider modifying `--asset-contains` filter or using a more powerful `--asset-regex-match`"
                            } else {
                                "multiple assets matched, consider using `--asset-contains` or `--asset-re` filter"
                            };

                            break 'outer Err(anyhow!(msg));
                        }
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
        // dbg!(tot_size);
        // dbg!(downloaded);

        Ok(temp_file_name)
    }
}
