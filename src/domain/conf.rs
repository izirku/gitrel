use super::package::PackageMap;
use crate::error::AppError;
use anyhow::Context;
use clap::{crate_name, ArgMatches};
use directories::{BaseDirs, ProjectDirs};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    gitrel: Gitrel,
    github_pagination: Pagination,
}

#[derive(Debug, Deserialize, Serialize)]
struct Gitrel {
    // maybe allow "cross-targeting" later
    // targes_os: Option<String>,
    // target_arch: Option<String>,
    // target_env: Option<String>,
    #[cfg(not(target_os = "windows"))]
    #[serde(default)]
    strip_execs: bool,
}

#[derive(Debug, Deserialize, Serialize)]
struct Pagination {
    #[serde(default = "gh_per_page_default")]
    per_page: usize,
    #[serde(default = "gh_max_pages_default")]
    max_pages: usize,
}

fn gh_per_page_default() -> usize {
    20
}

fn gh_max_pages_default() -> usize {
    5
}

pub struct ConfigurationManager {
    pub token: Option<String>,
    pub strip: bool,
    pub gh_per_page: usize,
    pub gh_max_pages: usize,
    pub bin_dir: PathBuf,
    // pub temp_dir: TempDir, // could use interior mutability to delay, but adds Arc<Mutex<Option<T>>> complexity
    packages: PathBuf,
}

impl ConfigurationManager {
    pub fn with_clap_matches(matches: &ArgMatches) -> Result<Self, AppError> {
        let base_dirs = BaseDirs::new().unwrap();
        let bin_dir = base_dirs.executable_dir().unwrap().to_path_buf();

        let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();
        let cfg_dir = proj_dirs.config_dir();
        fs::create_dir_all(cfg_dir)
            .with_context(|| format!("unable to create config dir: {:?}", cfg_dir))?;

        let config_file = cfg_dir.join("config.toml");
        let gh_token_file = cfg_dir.join("github_token.plain");
        let gh_ignore_file = cfg_dir.join(".gitignore");
        let packages = cfg_dir.join("packages.toml");

        let config = get_or_create_cofig_file(&config_file)?;
        ensure_gitignore(&gh_ignore_file)?;

        let token = match matches.value_of("token") {
            Some(token) => Some(token.to_owned()),
            None => gh_token_from_file(&gh_token_file),
        };

        // dbg!(&config);
        // dbg!(&token);

        Ok(ConfigurationManager {
            token,
            strip: config.gitrel.strip_execs,
            gh_per_page: config.github_pagination.per_page,
            gh_max_pages: config.github_pagination.max_pages,
            packages,
            bin_dir,
        })
    }

    pub fn get_packages(&self) -> Result<PackageMap, AppError> {
        match fs::read_to_string(self.packages.as_path()) {
            Ok(contents) => toml::from_str::<PackageMap>(&contents)
                .context(format!("malformed ackages TOML file: {:?}", self.packages))
                .map_err(AppError::AnyHow),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(AppError::NotFound),
            Err(e) => Err(AppError::AnyHow(
                anyhow::Error::new(e)
                    .context(format!("unable to read ackages file: {:?}", self.packages)),
            )),
        }
    }

    pub fn put_packages(&self, packages: &PackageMap) -> Result<(), AppError> {
        fs::write(
            self.packages.as_path(),
            toml::to_string(packages).context("parsing to toml")?,
        )
        .context("writing toml")?;
        Ok(())
    }
}

fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile, AppError> {
    match fs::read_to_string(&path) {
        Ok(config) => toml::from_str(&config)
            .with_context(|| format!("reading config: {:?}", path))
            .map_err(AppError::AnyHow),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = ConfigFile {
                gitrel: Gitrel {
                    // targes_os: Some(consts::OS.to_string()),
                    // target_arch: Some(consts::ARCH.to_string()),
                    #[cfg(not(target_os = "windows"))]
                    strip_execs: false,
                },
                github_pagination: Pagination {
                    per_page: gh_per_page_default(),
                    max_pages: gh_max_pages_default(),
                },
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
