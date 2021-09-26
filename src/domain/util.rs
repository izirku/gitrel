use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

lazy_static! {
    pub static ref SEMVER: Regex = Regex::new(r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?").expect("error parsing regex");

    static ref TERMS: Regex =
        Regex::new(r"(x86_64|x86\-64|32\-bit|[a-zA-Z0-9]+)").expect("error parsing regex");
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

/// Returns a triple (repo, repo_name, requested)
pub fn parse_gh_repo_spec(repo_spec: &str) -> (String, String, String) {
    let (repo, tag) = if repo_spec.contains('@') {
        let (repo, tag) = repo_spec.split_at(repo_spec.find('@').unwrap());
        (repo, tag.trim_start_matches('@'))
    } else {
        (repo_spec, "*")
    };

    let (repo, name) = if repo.contains('/') {
        (
            repo.to_owned(),
            repo.split_at(repo.find('/').unwrap())
                .1
                .get(1..)
                .unwrap()
                .to_lowercase(),
        )
    } else {
        (format!("{0}/{0}", repo), repo.to_lowercase())
    };

    (repo, name, tag.to_owned())
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
