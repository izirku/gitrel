pub mod requested;

use crate::foundation::consts;
use anyhow::{Context, Result};
use clap::{crate_name, ArgMatches};
use directories::BaseDirs;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use self::requested::PackageReqMap;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigFile {
    arch: Option<String>,
    os: Option<String>,
    bin_dir: Option<String>,
    strip: Option<bool>,
}

pub struct ConfigurationManager {
    pub token: Option<String>,
    requested: PathBuf,
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

        let config = get_or_create_cofig_file(&config_file)?;
        ensure_gitignore(&gh_ignore_file)?;

        let token = match matches.value_of("token") {
            Some(token) => Some(token.to_owned()),
            None => gh_token_from_file(&gh_token_file),
        };

        dbg!(config);
        dbg!(&token);

        Ok(ConfigurationManager { token, requested })
    }

    pub fn requested_packages(&self) -> Result<PackageReqMap> {
        let file = fs::read_to_string(self.requested.as_path())
            .with_context(|| format!("unable to read packages file: {:?}", self.requested))?;

        toml::from_str::<PackageReqMap>(&file)
            .with_context(|| format!("malformed packages TOML file: {:?}", self.requested))

        //     let requested = toml::from_str::<PackageReqMap>(&file)
        //         .with_context(|| format!("malformed packages TOML file: {:?}", self.requested))?;

        //     // let mut detailed_request_map = Vec::with_capacity(toml.len());
        //     let mut detailed_request_map: PackageReqDetailMap = BTreeMap::from(requested);

        //     for (name, pkg_spec) in toml.into_iter() {
        //         let pkg_spec = pkg_spec.into_detailed(&name);
        //         dbg!(&pkg_spec);
        //         let ver = format!("@ {}", &pkg_spec.matches);
        //         let repo = format!("[https://github.com/{}]", pkg_spec.repo.as_ref().unwrap());
        //         cols.push(vec![name, ver, repo]);
        //         dbg!(&pkg_spec);
        //     }
    }
}

fn get_or_create_cofig_file(path: &Path) -> Result<ConfigFile> {
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
