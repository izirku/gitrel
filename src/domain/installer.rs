use super::package::Package;
use super::util::{self, ArchiveKind, TarKind};
use crate::error::AppError;
use crate::Result;
use anyhow::Context;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;
#[cfg(target_family = "unix")]
use std::{
    fs::{set_permissions, Permissions},
    os::unix::fs::PermissionsExt,
};
use xz::read::XzDecoder;
use zip::ZipArchive;
// use tokio::fs::File;
// use tokio::io::{self, AsyncWriteExt};

pub async fn install(pkg: &Package, bin_dir: &Path) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(target_os="windows")] {
            let file_name = format!("{}.exe", pkg.name.as_ref().unwrap()).as_str();
        } else {
            let file_name = pkg.name.as_ref().unwrap().as_str();
        }
    }

    let archive_path = pkg.asset_path.as_ref().unwrap().as_path();
    let dest = bin_dir.join(file_name);
    let dest = dest.as_path();

    match util::archive_kind(pkg.asset_name.as_ref().unwrap()) {
        ArchiveKind::GZip => extract_gzip(archive_path, dest),
        ArchiveKind::BZip => extract_bzip(archive_path, dest),
        ArchiveKind::XZ => extract_xz(archive_path, dest),
        ArchiveKind::Zip => extract_zip(archive_path, file_name, dest),
        ArchiveKind::Tar(tar_kind) => extract_tar(archive_path, tar_kind, file_name, dest),
        ArchiveKind::Uncompressed => {
            let mut reader = BufReader::new(
                File::open(pkg.asset_path.as_ref().unwrap()).context("opening downloaded file")?,
            );
            let mut dest_file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(dest)
                .context("opening destination")?;

            match std::io::copy(&mut reader, &mut dest_file) {
                Ok(n) => {
                    println!("installed size: {}", n);
                    Ok(())
                }
                Err(e) => Err(AppError::AnyHow(
                    anyhow::Error::new(e).context("installing an uncompressed binary"),
                )),
            }
        }
        ArchiveKind::Unsupported => unreachable!(),
    }?;

    cfg_if::cfg_if! {
        if #[cfg(target_family = "unix")] {
            match set_permissions(dest, Permissions::from_mode(0o755)) {
                Ok(_) => Ok(()),
                Err(e) => Err(AppError::AnyHow(
                    anyhow::Error::new(e).context("setting the executable bit"),
                )),
            }
        } else {
            Ok(())
        }
    }
}

// TODO: maybe use flate2's tokio stuff?
fn extract_gzip(archive: &Path, dest: &Path) -> Result<(), AppError> {
    let mut reader = BufReader::new(GzDecoder::new(
        File::open(archive).context("opening a gzip file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context("opening destination")?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => {
            println!("decompressed bytes: {}", n);
            Ok(())
        }
        Err(e) => Err(AppError::AnyHow(
            anyhow::Error::new(e).context("decompressing a gzip file"),
        )),
    }
}

fn extract_bzip(archive: &Path, dest: &Path) -> Result<(), AppError> {
    let mut reader = BufReader::new(BzDecoder::new(
        File::open(archive).context("opening a bzip2 file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context("opening destination")?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => {
            println!("decompressed bytes: {}", n);
            Ok(())
        }
        Err(e) => Err(AppError::AnyHow(
            anyhow::Error::new(e).context("decompressing a bzip2 file"),
        )),
    }
}

fn extract_xz(archive: &Path, dest: &Path) -> Result<(), AppError> {
    let mut reader = BufReader::new(XzDecoder::new(
        File::open(archive).context("opening an xz file")?,
    ));
    let mut dest_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(dest)
        .context("opening destination")?;
    match std::io::copy(&mut reader, &mut dest_file) {
        Ok(n) => {
            println!("decompressed bytes: {}", n);
            Ok(())
        }
        Err(e) => Err(AppError::AnyHow(
            anyhow::Error::new(e).context("decompressing an xz file"),
        )),
    }
}

fn extract_zip(archive: &Path, file_name: &str, dest: &Path) -> Result<(), AppError> {
    let mut zip = ZipArchive::new(File::open(archive).context("opening a zip file")?)
        .context("reading a zip file")?;

    // first we have to find an index of what we want, without decompression
    let mut idx_to_extract = None;
    for i in 0..zip.len() {
        let zfile = zip.by_index_raw(i).context("indexing into a zip file")?;

        if let Some(zname) = zfile
            .enclosed_name()
            .and_then(Path::file_name)
            .and_then(OsStr::to_str)
        {
            if zname == file_name {
                idx_to_extract = Some(i);
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
            .context("opening destination")?;

        return match std::io::copy(&mut reader, &mut dest_file) {
            Ok(n) => {
                println!("decompressed bytes: {}", n);
                Ok(())
            }
            Err(e) => Err(AppError::AnyHow(
                anyhow::Error::new(e).context("decompressing a zip file"),
            )),
        };
    }
    Err(AppError::NotFound)
}

fn extract_tar(
    archive: &Path,
    tar_kind: TarKind,
    file_name: &str,
    dest: &Path,
) -> Result<(), AppError> {
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

    let reader = BufReader::new(File::open(tarball_path).context("reading a tarball")?);
    let mut tarball = tar::Archive::new(reader);

    for entry in tarball.entries().context("reading tarball entries")? {
        let mut entry = entry.context("reading a tarball entry")?;
        let entry_path = entry.path().context("getting a tarball entry path")?;
        if entry_path.file_name().and_then(OsStr::to_str).unwrap() == file_name {
            entry.unpack(dest).context("unpacking a tarball entry")?;
            return Ok(());
        }
    }

    Err(AppError::NotFound)
}
