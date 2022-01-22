use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::BufReader;
use std::path::Path;
#[cfg(target_family = "unix")]
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
};

use anyhow::{anyhow, Context, Result};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use xz::read::XzDecoder;
use zip::ZipArchive;

use super::util::{self, ArchiveKind, TarKind};

pub async fn install(
    repo: &str,
    asset_name: &str,
    asset_path: &Path,
    bin_dir: &Path,
    bin_name: &str,
    strip: bool,
    entry_contains: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let bin_name = util::bin_name(bin_name);
    let dest = bin_dir.join(&bin_name);
    let dest = dest.as_path();

    let bin_size = match util::archive_kind(asset_name) {
        ArchiveKind::GZip => extract_gzip(asset_path, dest),
        ArchiveKind::BZip => extract_bzip(asset_path, dest),
        ArchiveKind::XZ => extract_xz(asset_path, dest),
        ArchiveKind::Zip => extract_zip(asset_path, repo, dest, entry_contains, entry_re),
        ArchiveKind::Tar(tar_kind) => extract_tar(
            asset_path,
            tar_kind,
            repo,
            dest,
            entry_contains,
            entry_re,
        ),
        ArchiveKind::Uncompressed => {
            let mut reader =
                BufReader::new(File::open(asset_path).context("opening downloaded file")?);
            let mut dest_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(dest)
                .context(format!(
                    "{}:{}: {}",
                    file!(),
                    line!(),
                    "opening destination"
                ))?;

            match std::io::copy(&mut reader, &mut dest_file) {
                Ok(n) => Ok(n),
                Err(_e) => Err(anyhow!("installing an uncompressed binary")),
            }
        }
    }?;

    cfg_if::cfg_if! {
        if #[cfg(target_family = "unix")] {
            set_permissions(dest, Permissions::from_mode(0o755)).context("setting the execution bit")?;
            if strip {
                std::process::Command::new("strip").arg(dest).output().context("stripping the executable")?;
                let bin_size = fs::metadata(dest).context("getting installed binary metadata")?.len();
                Ok(bin_size)
            } else {
                Ok(bin_size)
            }
        } else {
            Ok(bin_size)
        }
    }
}

// TODO: maybe use flate2's tokio stuff?
fn extract_gzip(archive: &Path, dest: &Path) -> Result<u64> {
    let mut reader = BufReader::new(GzDecoder::new(
        File::open(archive).context("opening a gzip file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context(format!(
            "{}:{}: {}",
            file!(),
            line!(),
            "opening destination"
        ))?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => Ok(n),
        Err(_e) => Err(anyhow!("decompressing a gzip file")),
    }
}

fn extract_bzip(archive: &Path, dest: &Path) -> Result<u64> {
    let mut reader = BufReader::new(BzDecoder::new(
        File::open(archive).context("opening a bzip2 file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context(format!(
            "{}:{}: {}",
            file!(),
            line!(),
            "opening destination"
        ))?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => Ok(n),
        Err(_e) => Err(anyhow!("decompressing a bzip2 file")),
    }
}

fn extract_xz(archive: &Path, dest: &Path) -> Result<u64> {
    let mut reader = BufReader::new(XzDecoder::new(
        File::open(archive).context("opening an xz file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context(format!(
            "{}:{}: {}",
            file!(),
            line!(),
            "opening destination"
        ))?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => Ok(n),
        Err(_e) => Err(anyhow!("decompressing an xz file")),
    }
}

fn extract_zip(
    archive: &Path,
    file_name: &str,
    dest: &Path,
    entry_contains: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let mut zip = ZipArchive::new(File::open(archive).context("opening a zip file")?)
        .context("reading a zip file")?;

    // first we have to find an index of what we want, without decompression
    let mut idx_to_extract = None;
    for i in 0..zip.len() {
        let file_entry = zip.by_index_raw(i).context("indexing into a zip file")?;
        let file_entry = file_entry
            .enclosed_name()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str);

        if let Some(file_entry) = file_entry {
            if let Ok(res) = archive_entry_match(file_entry, file_name, entry_contains, entry_re) {
                if res {
                    idx_to_extract = Some(i);
                    break;
                }
            }
        }
    }

    // if we found something to decompress, roll with it
    if let Some(i) = idx_to_extract {
        let mut reader = BufReader::new(zip.by_index(i).context("indexing into a zip file")?);
        let mut dest_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(dest)
            .context(format!(
                "{}:{}: {}",
                file!(),
                line!(),
                "opening destination"
            ))?;

        return match std::io::copy(&mut reader, &mut dest_file) {
            Ok(n) => Ok(n),
            Err(_e) => Err(anyhow!("decompressing a zip file")),
        };
    }

    Err(anyhow!(format!(
        "binary `{}` not found inside the zip archive",
        file_name
    )))
}

fn extract_tar(
    archive: &Path,
    tar_kind: TarKind,
    file_name: &str,
    dest: &Path,
    entry_contains: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    // dbg!(&tar_kind);
    let tarball_path = match tar_kind {
        TarKind::GZip => {
            let uncompressed = archive.with_extension("");
            extract_gzip(archive, &uncompressed)?;
            // dbg!(&uncompressed);
            uncompressed
        }
        TarKind::BZip => {
            let uncompressed = archive.with_extension("");
            extract_bzip(archive, &uncompressed)?;
            uncompressed
        }
        TarKind::XZ => {
            let uncompressed = archive.with_extension("");
            extract_xz(archive, &uncompressed)?;
            uncompressed
        }
        TarKind::Uncompressed => archive.to_path_buf(),
    };

    let reader = BufReader::new(File::open(&tarball_path).context("reading a tarball")?);
    let mut tarball = tar::Archive::new(reader);

    for entry in tarball.entries().context("reading tarball entries")? {
        let mut entry = entry.context("reading a tarball entry")?;
        let file_entry = entry.path().context("getting a tarball entry path")?;
        let file_entry = file_entry.file_name().and_then(OsStr::to_str);

        if let Some(file_entry) = file_entry {
            if let Ok(res) = archive_entry_match(file_entry, file_name, entry_contains, entry_re) {
                if res {
                    entry.unpack(dest).context("unpacking a tarball entry")?;
                    let bin_size = fs::metadata(dest)
                        .context("getting installed binary metadata")?
                        .len();
                    return Ok(bin_size);
                }
            }
        }
    }

    Err(anyhow!(format!(
        "binary `{}` not found inside the tarball",
        file_name
    )))
}

fn archive_entry_match(
    archive_entry: &str,
    file_name: &str,
    entry_contains: Option<&str>,
    entry_re: Option<&str>,
) -> Result<bool> {
    if let Some(s) = entry_contains {
        Ok(archive_entry.contains(s))
    } else if let Some(s) = entry_re {
        let re = regex::Regex::new(s).context("invalid asset name RegEx expression")?;
        Ok(re.is_match(archive_entry))
    } else {
        Ok(archive_entry == file_name)
    }
}

// fn archive_entry_match(
//     file_name: String,
//     entry_contains: Option<String>,
//     entry_re: Option<String>,
// ) -> Result<Box<dyn Fn(&str) -> bool>> {
//     if let Some(re) = &entry_re {
//         let re = regex::Regex::new(re).context("invalid asset name RegEx expression")?;
//         Ok(Box::new(|archive_entry: &str| re.is_match(archive_entry)))
//     } else if let Some(s) = entry_contains {
//         Ok(Box::new(move |archive_entry: &str| {
//             archive_entry.contains(&s)
//         }))
//     } else {
//         Ok(Box::new(move |archive_entry: &str| {
//             archive_entry.contains(&file_name)
//         }))
//     }
// }
