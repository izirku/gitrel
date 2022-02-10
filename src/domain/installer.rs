use std::ffi::OsStr;
use std::fmt::Write;
use std::fs::{self, File, OpenOptions};
use std::io::BufReader;
use std::path::Path;
#[cfg(target_family = "unix")]
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
};

use anyhow::{anyhow, Context};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lazy_static::__Deref;
use xz::read::XzDecoder;
use zip::ZipArchive;

use super::error::InstallerError;
use super::util::{self, ArchiveKind, TarKind};

type Result<T, E = InstallerError> = std::result::Result<T, E>;

#[cfg(not(target_os = "windows"))]
pub async fn install(
    asset_name: &str,
    asset_path: &Path,
    bin_dir: &Path,
    bin_name: &str,
    strip: bool,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let dest = bin_dir.join(&bin_name);
    let dest = dest.as_path();

    let bin_size = match util::archive_kind(asset_name) {
        ArchiveKind::GZip => extract_gzip(asset_path, dest),
        ArchiveKind::BZip => extract_bzip(asset_path, dest),
        ArchiveKind::XZ => extract_xz(asset_path, dest),
        ArchiveKind::Zip => extract_zip(asset_path, bin_name, dest, entry_glob, entry_re),
        ArchiveKind::Tar(tar_kind) => {
            extract_tar(asset_path, tar_kind, bin_name, dest, entry_glob, entry_re)
        }
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
                Err(_e) => Err(InstallerError::AnyHow(anyhow!(
                    "installing an uncompressed binary"
                ))),
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

#[cfg(target_os = "windows")]
pub async fn install(
    asset_name: &str,
    asset_path: &Path,
    bin_dir: &Path,
    bin_name: &str,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let bin_name = format!("{}.exe", bin_name);
    let dest = bin_dir.join(&bin_name);
    let dest = dest.as_path();

    match util::archive_kind(asset_name) {
        ArchiveKind::GZip => extract_gzip(asset_path, dest),
        ArchiveKind::BZip => extract_bzip(asset_path, dest),
        ArchiveKind::XZ => extract_xz(asset_path, dest),
        ArchiveKind::Zip => extract_zip(asset_path, &bin_name, dest, entry_glob, entry_re),
        ArchiveKind::Tar(tar_kind) => {
            extract_tar(asset_path, tar_kind, &bin_name, dest, entry_glob, entry_re)
        }
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
                Err(_e) => Err(InstallerError::AnyHow(anyhow!(
                    "installing an uncompressed binary"
                ))),
            }
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
        Err(_e) => Err(InstallerError::AnyHow(anyhow!("decompressing a gzip file"))),
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
        Err(_e) => Err(InstallerError::AnyHow(anyhow!(
            "decompressing a bzip2 file"
        ))),
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
        Err(_e) => Err(InstallerError::AnyHow(anyhow!("decompressing an xz file"))),
    }
}

fn extract_zip(
    archive: &Path,
    file_name: &str,
    dest: &Path,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let mut zip = ZipArchive::new(File::open(archive).context("opening a zip file")?)
        .context("reading a zip file")?;
    let archive_entry_matcher = get_archive_entry_matcher(file_name, entry_glob, entry_re)?;
    let mut unmatched_entries = String::new();

    // first we have to find an index of what we want, without decompression
    let mut idx_to_extract = None;
    for i in 0..zip.len() {
        let file_entry = zip.by_index_raw(i).context("indexing into a zip file")?;
        let archive_entry = file_entry.enclosed_name();

        if let Some(archive_entry) = archive_entry {
            if archive_entry_matcher(archive_entry)? {
                idx_to_extract = Some(i);
                break;
            }
            writeln!(&mut unmatched_entries, "  {}", archive_entry.display())
                .map_err(anyhow::Error::msg)?;
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
            Err(_e) => Err(InstallerError::AnyHow(anyhow!("decompressing a zip file"))),
        };
    }

    Err(entry_match_error(
        archive.file_name().and_then(OsStr::to_str).unwrap(),
        file_name,
        entry_glob,
        entry_re,
        unmatched_entries,
    ))
}

fn extract_tar(
    archive: &Path,
    tar_kind: TarKind,
    file_name: &str,
    dest: &Path,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
) -> Result<u64> {
    let tarball_path = match tar_kind {
        TarKind::GZip => {
            let uncompressed = archive.with_extension("");
            extract_gzip(archive, &uncompressed)?;
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
    let archive_entry_matcher = get_archive_entry_matcher(file_name, entry_glob, entry_re)?;
    let mut unmatched_entries = String::new();

    for entry in tarball.entries().context("reading tarball entries")? {
        let mut entry = entry.context("reading a tarball entry")?;
        let archive_entry = entry.path().context("getting a tarball entry path")?;

        if archive_entry_matcher(archive_entry.deref())? {
            entry.unpack(dest).context("unpacking a tarball entry")?;
            let bin_size = fs::metadata(dest)
                .context("getting installed binary metadata")?
                .len();
            return Ok(bin_size);
        }

        writeln!(&mut unmatched_entries, "  {}", archive_entry.display())
            .map_err(anyhow::Error::msg)?;
    }

    Err(entry_match_error(
        archive.file_name().and_then(OsStr::to_str).unwrap(),
        file_name,
        entry_glob,
        entry_re,
        unmatched_entries,
    ))
}

fn entry_match_error(
    archive_name: &str,
    entry_exact: &str,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
    msg: String,
) -> InstallerError {
    if let Some(s) = entry_glob {
        InstallerError::EntryNotFound(s.to_owned(), "glob pattern", archive_name.to_owned(), msg)
    } else if let Some(s) = entry_re {
        InstallerError::EntryNotFound(s.to_owned(), "RegEx pattern", archive_name.to_owned(), msg)
    } else {
        InstallerError::EntryNotFound(
            entry_exact.to_owned(),
            "exact file name",
            archive_name.to_owned(),
            msg,
        )
    }
}

fn get_archive_entry_matcher(
    entry_exact: &str,
    entry_glob: Option<&str>,
    entry_re: Option<&str>,
) -> Result<Box<dyn Fn(&Path) -> Result<bool>>> {
    if let Some(s) = entry_glob {
        let glob = glob::Pattern::new(s).context("invalid asset name glob pattern")?;
        Ok(Box::new(move |archive_entry: &Path| {
            Ok(glob.matches_path(archive_entry))
        }))
    } else if let Some(s) = entry_re {
        let re = regex::Regex::new(s).context("invalid asset name RegEx pattern")?;
        Ok(Box::new(move |archive_entry: &Path| {
            Ok(re.is_match(&archive_entry.to_string_lossy()))
        }))
    } else {
        let entry_exact = entry_exact.to_owned();
        Ok(Box::new(move |archive_entry: &Path| {
            Ok(archive_entry.ends_with(&entry_exact))
        }))
    }
}
