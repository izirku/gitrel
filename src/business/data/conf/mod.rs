pub mod model;

use crate::foundation::consts;
use anyhow::{Context, Result};
use directories::BaseDirs;
use model::ConfigFile;
use std::fs;
use std::path::Path;

pub fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile> {
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
