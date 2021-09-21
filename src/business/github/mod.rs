mod asset;
mod release;
mod response;

pub use self::release::Release;
pub use self::response::GithubResponse;
use super::conf::{Package, PackageMatchKind};
use crate::business::conf::ConfigurationManager;
use crate::error::AppError;
use anyhow::Context;
use reqwest::{header, Client, Method};

pub struct GitHub {
    client: Client,
    per_page: usize,
    max_pages: usize,
}

impl GitHub {
    pub fn new(cm: &ConfigurationManager) -> Result<Self, anyhow::Error> {
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
            headers.insert(header::AUTHORIZATION, header::HeaderValue::from_str(token)?);
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .with_context(|| "creating REST API client has failed.")?;

        Ok(Self {
            client,
            per_page: cm.gh_per_page,
            max_pages: cm.gh_max_pages,
        })
    }

    // pub fn per_page(&mut self, per_page: u32) -> &mut Self {
    //     self.per_page = per_page;
    //     self
    // }

    pub async fn get_matching_release(&self, pkg: &Package) -> Result<Release, AppError> {
        let result = match pkg.match_kind() {
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

        result
    }

    async fn get_exact_release(&self, req_url: &str) -> Result<Release, AppError> {
        let resp = self
            .client
            .get(req_url)
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

        if let GithubResponse::Ok(release) = resp {
            Ok(release)
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
                .query(&[("page", curr_page)])
                .send()
                .await
                .with_context(|| "fething next page")?;

            dbg!(resp.status());

            if resp.status().as_u16() != 200 {
                return Err(AppError::NotFound);
            }

            let releases: Vec<GithubResponse<Release>> =
                resp.json().await.context("parsing response body")?;

            for release in releases.into_iter().filter_map(|resp| {
                if let GithubResponse::Ok(release) = resp {
                    Some(release)
                } else {
                    None
                }
            }) {
                if release.matches_semver(pkg)? {
                    break 'outer Ok(release);
                }
            }

            curr_page += 1;
            if curr_page > self.max_pages {
                break Err(AppError::NotFound);
            }
        }
    }
}
