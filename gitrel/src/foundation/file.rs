use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    path::Path,
};

pub fn gh_token_from_file(path: &Path) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(token) => Some(token.trim().to_string()),
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => None,
            _ => panic!("unable to read token file: {:?}: {}", path, err),
        },
    }
}

pub fn ensure_gitignore(path: &Path) -> Result<()> {
    match File::open(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            fs::write(path, "github_token.plain")
                .with_context(|| format!("unable to create file: {:?}", path))
        }
        Err(err) => Err(err).with_context(|| format!("unable to access file: {:?}", path)),
        _ => Ok(()),
    }
}
