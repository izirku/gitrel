use std::{collections::HashSet, fs, path::PathBuf};

use anyhow::{anyhow, Context, Result};
use console::style;
use directories::BaseDirs;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use regex::Regex;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

lazy_static! {
    pub static ref RX_REPO: Regex = Regex::new(r"(?:https://github\.com/)?((?:[^/]+/[^/]+)|[^/]+)(?:/)?").expect("error parsing regex");

    pub static ref SEMVER: Regex = Regex::new(r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?").expect("error parsing regex");

    static ref TERMS: Regex =
        Regex::new(r"(x86_64|x86\-64|32\-bit|[a-zA-Z0-9]+)").expect("error parsing regex");
}

pub fn message_fail(pb: &ProgressBar, repo_name: &str, msg: &str) {
    let msg = format!("{} {} {}", style('✗').red(), msg, style(&repo_name).red());
    pb.disable_steady_tick();
    pb.set_style(ProgressStyle::default_bar().template("{msg}"));
    pb.finish_with_message(msg);
}

pub fn matches_target(str: &str) -> bool {
    // `str` must not have any terms present in `EXCLUDE_SET`
    for term in TERMS.find_iter(&str.to_lowercase()) {
        if EXCLUDE_SET.contains(term.as_str()) {
            return false;
        }
    }
    // if include set is not empty, `str` must include at least 1 term
    // from the `INCLUDE_SET`
    if !INCLUDE_SET.is_empty() {
        // must include at least 1
        for term in TERMS.find_iter(&str.to_lowercase()) {
            if INCLUDE_SET.contains(term.as_str()) {
                return true;
            }
        }
        false
    } else {
        true
    }
}

pub fn matches_semver(tag_name: &str, semver: &str) -> bool {
    if let Some(extacted_remote_semver) = SEMVER.find(tag_name) {
        let ver_remote = semver::Version::parse(extacted_remote_semver.as_str());
        if let Ok(ver_remote) = ver_remote {
            let ver_req = semver::VersionReq::parse(semver).unwrap();
            return ver_req.matches(&ver_remote);
        }
    }
    false
}

#[derive(Debug, PartialEq)]
pub enum ArchiveKind {
    BZip,
    GZip,
    XZ,
    Zip,
    Tar(TarKind),
    Uncompressed,
}

#[derive(Debug, PartialEq)]
pub enum TarKind {
    Uncompressed,
    BZip,
    GZip,
    XZ,
}

pub fn archive_kind(str: &str) -> ArchiveKind {
    // order does matter
    if str.rfind('.').is_none() {
        ArchiveKind::Uncompressed
    } else if str.ends_with(".zip") {
        ArchiveKind::Zip
    } else if str.ends_with(".tar.gz") || str.ends_with(".tgz") {
        ArchiveKind::Tar(TarKind::GZip)
    } else if str.ends_with(".tar.bz2") || str.ends_with(".tbz") {
        ArchiveKind::Tar(TarKind::BZip)
    } else if str.ends_with(".tar.xz") || str.ends_with(".txz") {
        ArchiveKind::Tar(TarKind::XZ)
    } else if str.ends_with(".tar") {
        ArchiveKind::Tar(TarKind::Uncompressed)
    } else if str.ends_with(".gz") || str.ends_with(".gzip") || str.ends_with(".gnuzip") {
        ArchiveKind::GZip
    } else if str.ends_with(".bz2") || str.ends_with(".bzip2") {
        ArchiveKind::BZip
    } else if str.ends_with(".xz") {
        ArchiveKind::XZ
    } else {
        // NB: some names contain '.' in them, that are not necessary the archive extensions.
        // example: https://github.com/cloudfoundry/bosh-bootloader/releases/tag/v8.4.83
        // has an uncommpressed asset, named `bbl-v8.4.83-osx`.
        // Threfore, it's safer to treat such files as uncompressed, even though we might get
        // some exotic archiver
        ArchiveKind::Uncompressed
    }
}

/// Checks to see if `~/.config/gitrel/` directory exists, and then returns
/// the `~/.config/gitrel/packages.json` PathBuf.
pub fn packages_file() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("unable to get usable `base dir`"))?;
    let home_dir = base_dirs.home_dir();

    let cfg_dir = home_dir.join(".config/gitrel/");
    fs::create_dir_all(cfg_dir.as_path())
        .with_context(|| format!("unable to create config dir: {:?}", cfg_dir.as_path()))?;

    let path = cfg_dir.join("packages.json");

    Ok(path)
}

pub fn bin_dir() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("unable to get usable `base dir`"))?;
    let home_dir = base_dirs.home_dir();

    // if exists, prefer ~/.local/bin over ~/bin,
    // if DNE, create ~/.local/bin
    // (note: do it on all systems, even windows)
    let bin_dir = if home_dir.join(".local/bin/").exists() {
        home_dir.join(".local/bin/")
    } else if home_dir.join("bin/").exists() {
        home_dir.join("bin/")
    } else {
        let tmp = home_dir.join(".local/bin/");
        fs::create_dir_all(tmp.as_path()).context("create `~/.local/bin`")?;
        tmp
    };

    Ok(bin_dir)
}

/// Returns a tuple (user, repo, requested_version)
pub fn parse_gh_repo_spec(repo_spec: &str) -> Result<(String, String, String)> {
    // split [https://github.com/]user/repo@version at '@'
    let (repo_spec, requested) = if repo_spec.contains('@') {
        let (repo, requested) = repo_spec.split_at(repo_spec.find('@').unwrap());
        (repo, requested.trim_start_matches('@'))
    } else {
        (repo_spec, "*")
    };

    // capture the "user/repo" or the "repo"
    let caps = RX_REPO.captures(repo_spec).context("parsing repo name")?;
    let cap = caps.get(1).context("capturing repo name")?.as_str();

    // make sure we got the ("user/repo", "repo") tuple
    let (user, repo) = if cap.contains('/') {
        cap.split_once('/').unwrap()
    } else {
        (cap, cap)
    };

    Ok((user.to_owned(), repo.to_owned(), requested.to_owned()))
}
