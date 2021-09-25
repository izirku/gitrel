use super::package::Package;
use super::util::TarKind;
use crate::domain::util::{self, ArchiveKind};
use crate::error::AppError;
use crate::Result;
use anyhow::{anyhow, Context};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use std::ffi::OsStr;
#[cfg(target_family = "unix")]
use std::fs::{set_permissions, Permissions};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
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

    // let archive_path = pkg.asset_path.and_then(PathBuf::as_path).as_ref().unwrap();
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
            // buffered_reader_writer(&mut reader, &mut dest_file)
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
    // buffered_reader_writer(&mut reader, &mut dest_file)
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
    // buffered_reader_writer(&mut reader, &mut dest_file)
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
    // buffered_reader_writer(&mut reader, &mut dest_file)
}

fn extract_zip(archive: &Path, file_name: &str, dest: &Path) -> Result<(), AppError> {
    let mut zip = ZipArchive::new(File::open(archive).context("opening a zip file")?)
        .context("reading a zip file")?;

    for i in 0..zip.len() {
        let zfile = zip.by_index_raw(i).context("indexing into a zip file")?;

        if let Some(zpath) = zfile.enclosed_name() {
            if let Some(zname) = zpath.file_name() {
                if let Some(zname) = zname.to_str() {
                    if zname == file_name {
                        let mut dest_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(dest)
                            .context("opening destination")?;
                        let mut reader = BufReader::new(zfile);
                        return buffered_reader_writer(&mut reader, &mut dest_file);
                    }
                }
            }
        }
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

fn buffered_reader_writer<R: BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
) -> Result<(), AppError> {
    let mut writer = BufWriter::new(writer);
    loop {
        let buffer = reader
            .fill_buf()
            .context("failed to fill the read buffer")?;

        if buffer.is_empty() {
            break;
        }

        match writer.write(buffer) {
            Ok(0) => {
                return Err(AppError::AnyHow(anyhow!(
                    "unable to write to dest buffer anymore"
                )))
            }
            Ok(n) => {
                reader.consume(n);
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => {
                return Err(AppError::AnyHow(
                    anyhow::Error::new(e).context("writing to the dest buffer"),
                ))
            }
        }
    }
    Ok(())
}

//                         let mut writer = BufWriter::new(dest_file);
//                         loop {
//                             let buffer = reader
//                                 .fill_buf()
//                                 .context("failed to fill the read buffer")?;

//                             if buffer.len() == 0 {
//                                 break;
//                             }

//                             match writer.write(buffer) {
//                                 Ok(0) => {
//                                     return Err(AppError::AnyHow(anyhow!(
//                                         "unable to write to dest buffer anymore"
//                                     )))
//                                 }
//                                 Ok(n) => {
//                                     reader.consume(n);
//                                     continue;
//                                 }
//                                 Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
//                                 Err(e) => {
//                                     return Err(AppError::AnyHow(
//                                         anyhow::Error::new(e).context("writing to the dest buffer"),
//                                     ))
//                                 }
//                             }
//                         }
//                         // zip.extract_file(i, &dest, true).context("extracting a file from a zip")?;
//                         return Ok(());
