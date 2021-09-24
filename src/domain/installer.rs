use super::conf::Package;
use crate::Result;
use std::path::Path;
// #[cfg(target_family = "unix")]
// use std::fs::{set_permissions, Permissions};
// #[cfg(target_family = "unix")]
// use std::os::unix::fs::PermissionsExt;
// use tokio::fs::File;
// use tokio::io::{self, AsyncWriteExt};

pub async fn _extract(_pkg: &Package, _bin_dir: &Path) -> Result<(), anyhow::Error> {
    Ok(())
}
