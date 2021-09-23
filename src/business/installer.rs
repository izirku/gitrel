use super::conf::{ConfigurationManager, Package};
use super::github::Release;
use crate::error::AppError;
use crate::Result;
use anyhow::Context;
use reqwest::Client;
#[cfg(target_family = "unix")]
use std::fs::{set_permissions, Permissions};
use std::hash::{Hash, Hasher};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
// use tempfile::{self, tempdir, tempfile};
use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};

pub struct Installer<'a> {
    client: Client,
    temp_dir: TempDir,
    bin_path: &'a Path,
}

impl<'a> Installer<'a> {
    pub fn new(cm: &'a ConfigurationManager) -> Result<Self, anyhow::Error> {
        use reqwest::header;
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/octet-stream"),
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
            .context("creating REST API client has failed.")?;

        let temp_dir = tempfile::tempdir().context("creating temp dir")?;
        Ok(Self {
            client,
            temp_dir,
            bin_path: cm.bin_dir.as_path(),
        })
    }

    pub async fn download(&self, repo: &str, asset_id: &str) -> Result<PathBuf, AppError> {
        use anyhow::anyhow;
        use reqwest::StatusCode;
        let req_url = format!(
            "https://api.github.com/repos/{}/releases/assets/{}",
            repo, asset_id
        );
        let mut resp = self
            .client
            .get(&req_url)
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

        // let mut hasher = DefaultHasher::new();
        let mut hasher = twox_hash::XxHash64::default();
        repo.hash(&mut hasher);
        let temp_file_name = format!("{:x}", hasher.finish());
        let temp_file_name = self.temp_dir.path().join(&temp_file_name);
        let mut temp_file = File::create(temp_file_name.as_path())
            .await
            .context(format!(
                "creating a temp file: {:?}",
                temp_file_name.as_path(),
            ))?;
        // let temp_file = tempfile::tempfile().context("creating a temp file")?;

        while let Some(chunk) = resp.chunk().await.context("retrieving a next chunk")? {
            temp_file
                .write_all(&chunk)
                .await
                .context("writing a chunk to temp file")?;
        }
        println!("temp file created: {:?}", &temp_file_name);

        Ok(temp_file_name)
    }

    pub async fn extract(&self, archive: &Path, asset_name: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    // pub async fn run(&self, pkg: &Package,  )
}
