use super::package::Package;
use anyhow::{anyhow, Context, Result};
use directories::BaseDirs;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

pub type PackageMap = BTreeMap<String, Package>;

pub struct Packages {
    location: PathBuf,
}

impl Packages {
    pub fn new() -> Result<Self> {
        let base_dirs =
            BaseDirs::new().ok_or_else(|| anyhow!("unable to get usable `base dir`"))?;
        let home_dir = base_dirs.home_dir();

        // let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();
        // let cfg_dir = proj_dirs.config_dir();
        let cfg_dir = home_dir.join(".config/gitrel/");
        fs::create_dir_all(cfg_dir.as_path())
            .with_context(|| format!("unable to create config dir: {:?}", cfg_dir.as_path()))?;

        let path = cfg_dir.join("packages.toml");

        Ok(Self { location: path })
    }

    pub fn get(&self) -> Result<Option<PackageMap>> {
        match fs::read_to_string(self.location.as_path()) {
            Ok(contents) => Ok(Some(
                toml::from_str::<PackageMap>(&contents)
                    .context(format!("malformed packages TOML file: {:?}", self.location))?,
            )),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_e) => Err(anyhow!(format!(
                "unable to read packages file: {:?}",
                self.location
            ))),
        }
    }

    pub fn put(&self, packages: &PackageMap) -> Result<()> {
        fs::write(
            self.location.as_path(),
            toml::to_string(packages).context("parsing to toml")?,
        )
        .context("writing toml")?;
        Ok(())
    }
}
