use anyhow::{anyhow, Context, Result};
use console::style;
use directories::BaseDirs;
use indicatif::{ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashSet, fs, path::PathBuf};
use url::Url;

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
    for term in TERMS.find_iter(&str.to_lowercase()) {
        if EXCLUDE_SET.contains(term.as_str()) {
            return false;
        }
    }
    true
    //     rx::MATCH_OS.is_match(&self.name)
    //         && rx::MATCH_ARCH.is_match(&self.name)
    //         && rx::MATCH_ABI.is_match(&self.name)
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

// pub fn match_asset() {
//     let selection: Vec<_> = release.assets.iter().map(|asset| &asset.name).collect();
//     // dbg!(selection);
//     let msel = Select::with_theme(&ColorfulTheme::default())
//         .with_prompt("Multiple results found, select one")
//         .items(&selection)
//         .interact()
//         .unwrap();
//     dbg!(msel);
// }

#[derive(Debug, PartialEq)]
pub enum ArchiveKind {
    BZip,
    GZip,
    XZ,
    Zip,
    Tar(TarKind),
    Uncompressed,
    Unsupported,
}

#[derive(Debug, PartialEq)]
pub enum TarKind {
    Uncompressed,
    BZip,
    GZip,
    XZ,
}

pub fn archive_kind(str: &str) -> ArchiveKind {
    if str.rfind('.').is_none() {
        return ArchiveKind::Uncompressed;
    }

    // order does matter
    if str.ends_with(".zip") {
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
        ArchiveKind::Unsupported
    }
}

/// Checks to see if `~/.config/gitrel/` directory exists, and then returns
/// the `~/.config/gitrel/packages.json` PathBuf.
pub fn packages_file() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("unable to get usable `base dir`"))?;
    let home_dir = base_dirs.home_dir();

    // let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();
    // let cfg_dir = proj_dirs.config_dir();
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

pub fn bin_name(repo_url: &Url) -> String {
    cfg_if::cfg_if! {
        if #[cfg(target_os="windows")] {
            format!("{}.exe", repo_name(repo_url))
        } else {
            repo_name(repo_url)
        }
    }
}

pub fn repo_name(repo_url: &Url) -> String {
    let (_, repo_name) = repo_url
        .path()
        .split_at(repo_url.path().rfind('/').unwrap());
    let repo_name = repo_name.trim_start_matches('/');
    repo_name.to_lowercase()
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
    // let cap = cap.as_str();

    // make sure we got the ("user/repo", "repo") tuple
    let (user, repo) = if cap.contains('/') {
        cap.split_once('/').unwrap()
    } else {
        (cap, cap)
    };

    Ok((user.to_owned(), repo.to_owned(), requested.to_owned()))

    // let (url, repo_name)  = match Url::parse(repo) {
    //     Ok(url) => {
    //         let splits: Vec<_> = url.path().split_terminator('/').filter(|s| !s.is_empty()).collect();
    //         if splits.len() != 2 {
    //             return Err(anyhow!("incorrect repo url: {}", url.as_str()));
    //         }
    //         (url, splits[1])
    //     }
    //     Err(_) => {
    //         let splits: Vec<_> = repo.split_terminator('/').filter(|s| !s.is_empty()).collect();
    //         if splits.len() != 2 {
    //             return Err(anyhow!("incorrect repo: {}", url.as_str()));
    //         }
    //         let url = Url::parse("https://github.com/").and_then(|url| {
    //             if repo.contains('/') {
    //                 url.join(repo)
    //             } else {
    //                 url.join(format!("{0}/{0}", repo).as_str())
    //             }
    //         };
    //         (url, splits[1])
    //     }
    // };

    // let repo = Url::parse(repo)
    //     .or_else(|_| {
    //         Url::parse("https://github.com/").and_then(|url| {
    //             if repo.contains('/') {
    //                 url.join(repo)
    //             } else {
    //                 url.join(format!("{0}/{0}", repo).as_str())
    //             }
    //         })
    //     })
    //     .unwrap();

    // let (repo, name) = if repo.contains('/') {
    //     (
    //         repo.to_owned(),
    //         repo.split_at(repo.find('/').unwrap())
    //             .1
    //             .get(1..)
    //             .unwrap()
    //             .to_lowercase(),
    //     )
    // } else {
    //     (format!("{0}/{0}", repo), repo.to_lowercase())
    // };

    // (repo, name, requested.to_owned())
    // (repo, requested.to_owned())
}

// this is imlemented as a Package method right now... should we move it out to here?
// #[derive(Debug)]
// pub enum MatchKind {
//     Exact,
//     Latest,
//     SemVer,
// }

// pub fn match_kind(str: &str) -> MatchKind {
//     if str == "*" {
//         MatchKind::Latest
//     } else if semver::VersionReq::parse(str).is_ok() {
//         MatchKind::SemVer
//     } else {
//         MatchKind::Exact
//     }
// }
