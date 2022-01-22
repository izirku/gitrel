use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn uninstall(bin_name: &str, bin_dir: &Path) -> Result<()> {
    let file_to_delete = bin_dir.join(bin_name);
    fs::remove_file(&file_to_delete)
        .context(format!("deleting a binary: {}", file_to_delete.display()))?;
    Ok(())
}
