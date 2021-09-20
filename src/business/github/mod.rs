mod model;

pub use self::model::{GithubResponse, Release};
use super::conf::{Package, PackageMatchKind};
use crate::error::AppError;
use crate::business::conf::ConfigurationManager;
use anyhow::Context;
use reqwest::{header, Client, Method};

pub struct GitHub {
    client: Client,
    per_page: u32,
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
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(token)?,
            );
        }

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .with_context(|| "creating REST API client has failed.")?;

        Ok(Self {
            client,
            per_page: 20,
            max_pages: cm.gh_pagination_max,
        })
    }

    pub fn per_page(&mut self, per_page: u32) -> &mut Self {
        self.per_page = per_page;
        self
    }

    pub async fn get_matching_release(
        &self,
        pkg: &Package<'_>,
    ) -> Result<GithubResponse<Release>, AppError> {
        match pkg.match_kind {
            PackageMatchKind::Latest => self.get_latest_release(pkg).await,
            PackageMatchKind::Exact => self.get_release_by_tag(pkg).await,
            PackageMatchKind::SemVer => self.find_release(pkg).await,
            PackageMatchKind::Unknown => Err(AppError::UnknownMatchKind),
        }
    }

    async fn find_release(&self, pkg: &Package<'_>) -> Result<GithubResponse<Release>, AppError> {
        let req_url = format!(
            "https://api.github.com/repos/{}/releases?per_page={}",
            pkg.repo().unwrap(),
            self.per_page,
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

            let releases: Vec<Release> = resp.json().await.context("parsing response body")?;

            for release in releases {
                if release.matches(pkg)? {
                    break 'outer Ok(GithubResponse::Ok(release));
                }
            }

            curr_page += 1;
            if curr_page > self.max_pages {
                break Err(AppError::NotFound);
            }
        }
    }

    async fn get_latest_release(
        &self,
        pkg: &Package<'_>,
    ) -> Result<GithubResponse<Release>, AppError> {
        // dbg!(pkg);

        let req_url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            pkg.repo().unwrap()
        );

        let resp = self
            .client
            .get(&req_url)
            .send()
            .await
            .context("fething latest release")?;

        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound);
        }

        resp.json::<GithubResponse<Release>>()
            .await
            .context("parsing latest release response body")
            .map_err(AppError::AnyHow)
    }

    async fn get_release_by_tag(
        &self,
        pkg: &Package<'_>,
    ) -> Result<GithubResponse<Release>, AppError> {
        // dbg!(pkg);

        let req_url = format!(
            "https://api.github.com/repos/{}/releases/tags/{}",
            pkg.repo().unwrap(),
            pkg.requested.unwrap().version,
        );

        let resp = self
            .client
            .get(&req_url)
            .send()
            .await
            .context("fething a release")?;

        if resp.status().as_u16() == 404 {
            return Err(AppError::NotFound);
        }

        resp.json::<GithubResponse<Release>>()
            .await
            .context("parsing release response body")
            .map_err(AppError::AnyHow)
    }
}
