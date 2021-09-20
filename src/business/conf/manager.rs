use super::installed::InstalledPackageMap;
use super::requested::PackageReqMap;
use crate::error::AppError;
use crate::foundation::consts;
use anyhow::Context;
use clap::{crate_name, ArgMatches};
use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    arch: Option<String>,
    os: Option<String>,
    bin_dir: Option<String>,
    #[serde(default)]
    strip: bool,
    #[serde(default = "gh_pagination_per_page_default")]
    pub gh_pagination_per_page: usize,
    #[serde(default = "gh_pagination_max_default")]
    gh_pagination_max: usize,
}

fn gh_pagination_per_page_default() -> usize {
    20
}

fn gh_pagination_max_default() -> usize {
    5
}

pub struct ConfigurationManager {
    pub token: Option<String>,
    pub strip: bool,
    pub gh_pagination_per_page: usize,
    pub gh_pagination_max: usize,
    requested: PathBuf,
    installed: PathBuf,
}

impl ConfigurationManager {
    pub fn with_clap_matches(matches: &ArgMatches) -> Result<Self, AppError> {
        let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();

        let cfg_dir = proj_dirs.config_dir();
        fs::create_dir_all(cfg_dir)
            .with_context(|| format!("unable to create config dir: {:?}", cfg_dir))?;

        let config_file = cfg_dir.join("config.toml");
        let gh_token_file = cfg_dir.join("github_token.plain");
        let gh_ignore_file = cfg_dir.join(".gitignore");
        let requested = cfg_dir.join("requested.toml");
        let installed = cfg_dir.join("installed.toml");

        let config = get_or_create_cofig_file(&config_file)?;
        ensure_gitignore(&gh_ignore_file)?;

        let token = match matches.value_of("token") {
            Some(token) => Some(token.to_owned()),
            None => gh_token_from_file(&gh_token_file),
        };

        dbg!(&config);
        dbg!(&token);

        Ok(ConfigurationManager {
            token,
            strip: config.strip,
            gh_pagination_per_page: config.gh_pagination_per_page,
            gh_pagination_max: config.gh_pagination_max,
            requested,
            installed,
        })
    }

    pub fn get_requested_packages(&self) -> Result<PackageReqMap, AppError> {
        match fs::read_to_string(self.requested.as_path()) {
            Ok(contents) => toml::from_str::<PackageReqMap>(&contents)
                .context(format!(
                    "malformed requested packages TOML file: {:?}",
                    self.requested
                ))
                .map_err(AppError::AnyHow),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::NotFound),
            Err(e) => Err(AppError::AnyHow(anyhow::Error::new(e).context(format!(
                "unable to read requested packages file: {:?}",
                self.requested
            )))),
        }
    }

    pub fn get_installed_packages(&self) -> Result<InstalledPackageMap, AppError> {
        match fs::read_to_string(self.installed.as_path()) {
            Ok(contents) => toml::from_str::<InstalledPackageMap>(&contents)
                .context(format!(
                    "malformed installed packages TOML file: {:?}",
                    self.installed
                ))
                .map_err(AppError::AnyHow),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::NotFound),
            Err(e) => Err(AppError::AnyHow(anyhow::Error::new(e).context(format!(
                "unable to read installed packages file: {:?}",
                self.installed
            )))),
        }
    }

    pub fn put_installed_packages(
        &self,
        installed_packages: &InstalledPackageMap,
    ) -> Result<(), AppError> {
        fs::write(
            self.installed.as_path(),
            toml::to_string(installed_packages).context("parsing to toml")?,
        )
        .context("writing toml")?;
        Ok(())
    }
}

fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile, AppError> {
    let base_dirs = BaseDirs::new().unwrap();
    let bin_dir = base_dirs.executable_dir().unwrap().to_string_lossy();

    match fs::read_to_string(&path) {
        Ok(config) => toml::from_str(&config)
            .with_context(|| format!("reading config: {:?}", path))
            .map_err(AppError::AnyHow),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = ConfigFile {
                os: Some(consts::OS.to_string()),
                arch: Some(consts::ARCH.to_string()),
                bin_dir: Some(bin_dir.to_string()),
                strip: false,
                gh_pagination_per_page: gh_pagination_per_page_default(),
                gh_pagination_max: gh_pagination_max_default(),
            };

            fs::write(&path, toml::to_string(&config).context("parsing to toml")?)
                .context("writing toml")?;
            Ok(config)
        }
        Err(e) => Err(AppError::AnyHow(
            anyhow::Error::new(e).context(format!("unable to read config file: {:?}", path)),
        )),
    }
}

fn gh_token_from_file(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(token) => Some(token.trim().to_string()),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => None,
            _ => panic!("unable to read token file: {:?}: {}", path, err),
        },
    }
}

fn ensure_gitignore(path: &Path) -> Result<(), AppError> {
    match File::open(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, "github_token.plain")
                .context(format!("unable to create file: {:?}", path))
                .map_err(AppError::AnyHow)
        }
        Err(e) => Err(AppError::AnyHow(
            anyhow::Error::new(e).context(format!("unable to access file: {:?}", path)),
        )),
        _ => Ok(()),
    }
}
