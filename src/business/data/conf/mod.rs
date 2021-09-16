pub mod model;

use anyhow::{Context, Result};
use clap::{crate_name, ArgMatches};
use directories::ProjectDirs;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

// use directories::BaseDirs;
// use regex::Regex;
// use chrono::prelude::*;
// use crate::foundation::consts;
// use semver::{Version, VersionReq};
// use serde::{Deserialize, Serialize};
// use std::{cmp::Ordering, collections::BTreeMap};

pub struct ConfigurationManager {
    pub token: Option<String>,
    pub requested: PathBuf,
}

impl ConfigurationManager {
    pub fn with_clap_matches(matches: &ArgMatches) -> Result<Self> {
        let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();

        let cfg_dir = proj_dirs.config_dir();
        fs::create_dir_all(cfg_dir)
            .with_context(|| format!("unable to create config dir: {:?}", cfg_dir))?;

        let config_file = cfg_dir.join("config.toml");
        let gh_token_file = cfg_dir.join("github_token.plain");
        let gh_ignore_file = cfg_dir.join(".gitignore");
        let requested = cfg_dir.join("requested.toml");

        let config = model::get_or_create_cofig_file(&config_file)?;
        ensure_gitignore(&gh_ignore_file)?;

        let token = match matches.value_of("token") {
            Some(token) => Some(token.to_owned()),
            None => gh_token_from_file(&gh_token_file),
        };

        dbg!(config);
        dbg!(&token);

        Ok(ConfigurationManager { token, requested })
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

fn ensure_gitignore(path: &Path) -> Result<()> {
    match File::open(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, "github_token.plain")
                .with_context(|| format!("unable to create file: {:?}", path))
        }
        Err(err) => Err(err).with_context(|| format!("unable to access file: {:?}", path)),
        _ => Ok(()),
    }
}
