mod asset;
mod release;
mod response;

use std::cmp;
// use std::fmt::Write;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use indicatif::{ProgressBar, ProgressStyle};
use tempfile::TempDir;
use ureq::{Agent, AgentBuilder};

use crate::domain::github::response::ErrorResponse;

use self::release::Release;
use self::response::GithubResponse;
use super::error::GithubError;
use super::package::{match_kind, Package, PackageMatchKind};
use super::util;

type Result<T, E = GithubError> = std::result::Result<T, E>;

const GH_MAX_PAGES: usize = 5;
const GH_PER_PAGE: usize = 25;

pub struct GitHub<'a> {
    agent: Agent,
    token: Option<&'a str>, // api_headers: Vec<(&'a str, String)>,
                            // dl_headers: Vec<(&'a str, String)>,
}

impl<'a> GitHub<'a> {
    pub fn create(token: Option<&'a str>) -> Self {
        // let mut api_headers = vec![("Accept", "application/vnd.github.v3+json".to_owned())];

        // let mut dl_headers = vec![("Accept", "application/octet-stream".to_owned())];

        // if let Some(token) = token {
        //     let token = format!("token {}", token);

        //     api_headers.push(("Authorization", token.to_owned()));
        // }

        let agent = AgentBuilder::new()
            .timeout_read(std::time::Duration::from_secs(5))
            .timeout_write(std::time::Duration::from_secs(5))
            .build();

        Self { agent, token }
    }

    /// Find a `Release` matching provided parameters.
    pub fn find_new(
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
                self.find_release_exact(&req_url, repo, asset_glob, asset_re)
            }
            PackageMatchKind::Exact => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases/tags/{}",
                    user, repo, requested,
                );
                self.find_release_exact(&req_url, repo, asset_glob, asset_re)
            }
            PackageMatchKind::SemVer => {
                let req_url = format!(
                    "https://api.github.com/repos/{}/{}/releases?per_page={}",
                    user, repo, GH_PER_PAGE,
                );
                self.find_release(&req_url, requested, repo, asset_glob, asset_re)
            }
        }
    }

    /// Find a `Release` matching provided `Package`.
    /// When `force` is `true`, return `Release`, even if it's not newer than
    /// the one specified in `Package`
    pub fn find_existing(&self, package: &Package) -> Result<Release> {
        let res = self.find_new(
            &package.user,
            &package.repo,
            &package.requested,
            package.asset_glob.as_deref(),
            package.asset_re.as_deref(),
        );

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

    fn find_release_exact(
        &self,
        req_url: &str,
        repo: &str,
        asset_glob: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Release> {
        // use reqwest::StatusCode;

        // let mut api_headers = vec![("Accept", "application/vnd.github.v3+json".to_owned())];

        // let mut dl_headers = vec![("Accept", "application/octet-stream".to_owned())];

        // if let Some(token) = token {
        //     let token = format!("token {}", token);

        //     api_headers.push(("Authorization", token.to_owned()));
        // }

        let req = self.agent.get(req_url);

        let req = if let Some(token) = self.token {
            req.set("Authorization", token)
        } else {
            req
        };

        let resp = req
            .set("Accept", "application/vnd.github.v3+json")
            .call()
            .context("fetching latest release")?;

        if resp.status() == 404 {
            return Err(GithubError::ReleaseNotFound);
        }

        if resp.status() != 200 {
            return Err(GithubError::AnyHow(anyhow!("getting")));
        }

        let resp: GithubResponse<Release> = resp
            .into_json()
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

    fn find_release(
        &self,
        req_url: &str,
        requested: &str,
        repo: &str,
        asset_glob: Option<&str>,
        asset_re: Option<&str>,
    ) -> Result<Release> {
        // use reqwest::StatusCode;
        let asset_matcher = get_asset_name_matcher(repo, asset_glob, asset_re)?;
        let mut curr_page: usize = 1;

        let req = self
            .agent
            .get(req_url)
            .set("Accept", "application/vnd.github.v3+json");

        let req = if let Some(token) = self.token {
            req.set("Authorization", token)
        } else {
            req
        };

        // let resp = req
        //     .set("Accept", "application/vnd.github.v3+json")
        //     .call()
        //     .context("fetching latest release")?;

        'outer: loop {
            let resp = req
                .query("page", &curr_page.to_string())
                .call()
                .context("sending request")?;

            if resp.status() == 404 {
                return Err(GithubError::ReleaseNotFound);
            }

            if resp.status() != 200 {
                return Err(GithubError::AnyHow(anyhow!("getting")));
            }

            let releases: GithubResponse<Vec<Release>> =
                resp.into_json().context("parsing response body")?;

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

    pub fn download(
        &self,
        user: &str,
        repo: &str,
        asset_id: u64,
        asset_name: &str,
        temp_dir: &TempDir,
    ) -> Result<PathBuf> {
        // use reqwest::StatusCode;
        let req_url = format!(
            "https://api.github.com/repos/{}/{}/releases/assets/{}",
            user, repo, asset_id
        );

        let req = self.agent.get(&req_url);

        let req = if let Some(token) = self.token {
            req.set("Authorization", token)
        } else {
            req
        };

        let resp = req
            .set("Accept", "application/octet-stream")
            .call()
            .context("fetching an asset")?;

        if resp.status() == 404 {
            return Err(GithubError::AssetNotFound);
        }

        if resp.status() != 200 {
            let mut msg = format!("getting: {}", &req_url);
            if let Ok(txt) = resp.into_string() {
                msg.push('\n');
                msg.push_str(&txt);
            }
            return Err(GithubError::AnyHow(anyhow!(msg)));
        }

        let tot_size = resp
            .header("Content-Length")
            .context("getting content length")?
            .parse::<u64>()
            .context("parsing content length")?;

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::with_template("{spinner:.blue} {msg}")
                .unwrap()
                .tick_strings(&[
                    "▹▹▹▹▹",
                    "▸▹▹▹▹",
                    "▹▸▹▹▹",
                    "▹▹▸▹▹",
                    "▹▹▹▸▹",
                    "▹▹▹▹▸",
                    "▪▪▪▪▪",
                ]),
        );

        let msg = format!(
            "downloading {} ({})",
            asset_name,
            bytesize::to_string(tot_size, false)
        );

        pb.set_message(msg);

        let temp_file_name = temp_dir.path().join(asset_name);
        let mut temp_file = File::create(temp_file_name.as_path()).context(format!(
            "creating a temp file: {:?}",
            temp_file_name.as_path(),
        ))?;

        let resp_reader = resp.into_reader();
        let mut buf = [0u8; 4096];

        // TODO: this needs to be looked at closer
        loop {
            match resp_reader
                .read(&mut buf)
                .context("reading response bytes")?
            {
                0 => break,
                n => temp_file
                    .write_all(&mut buf[0..n])
                    .context("writing to file")?,
            }
        }

        // temp_file.write()
        // while let Some(item) = resp_reader.read_vectored

        // while let Some(item) = stream.next().await {
        //     let chunk = item.context("retrieving a next chunk")?;
        //     temp_file
        //         .write(&chunk)
        //         .await
        //         .context("writing a chunk to temp file")?;
        //     let new = cmp::min(downloaded + (chunk.len() as u64), tot_size);
        //     downloaded = new;
        //     pb.set_position(new);
        // }

        // pb.finish_and_clear();
        pb.finish();

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
