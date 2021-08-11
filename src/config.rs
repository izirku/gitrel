use crate::consts;
use crate::models::config::ConfigFile;
use anyhow::{Context, Result};
use directories::BaseDirs;
use std::{
    fs::{self, File},
    path::Path,
};

pub(crate) fn gh_token_from_file(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(token) => Some(token.trim().to_string()),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => None,
            _ => panic!("unable to read token file: {:?}: {}", path, err),
        },
    }
}

pub(crate) fn ensure_gitignore(path: &Path) -> Result<()> {
    match File::open(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, "github_token.plain")
                .with_context(|| format!("unable to create file: {:?}", path))
        }
        Err(err) => Err(err).with_context(|| format!("unable to access file: {:?}", path)),
        _ => Ok(()),
    }
}

pub(crate) fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile> {
    let base_dirs = BaseDirs::new().unwrap();
    let bin_dir = base_dirs.executable_dir().unwrap().to_string_lossy();

    match fs::read_to_string(&path) {
        Ok(config) => {
            toml::from_str(&config).with_context(|| format!("reading config: {:?}", path))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let config = ConfigFile {
                os: Some(consts::OS.to_string()),
                arch: Some(consts::ARCH.to_string()),
                bin_dir: Some(bin_dir.to_string()),
                strip: Some(true),
            };

            fs::write(&path, toml::to_string(&config)?)?;
            Ok(config)
        }
        Err(err) => Err(err).with_context(|| format!("unable to read config file: {:?}", path)),
    }
}
