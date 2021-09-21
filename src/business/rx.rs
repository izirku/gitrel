use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    // source: https://regex101.com/r/Ly7O1x/1070
    pub static ref SEMVER: Regex = Regex::new(r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?").expect("error parsing regex");
}

// =================================================================================
// MATCH_OS

// These values are those supported by Rust (based on the platforms
// crate) and Go (based on
// https://gist.github.com/asukakenji/f15ba7e588ac42795f421b48b8aede63).

#[cfg(target_os = "linux")]
lazy_static! {
    pub static ref MATCH_OS: Regex =
        Regex::new(r"(?i:(?:\b|_)linux(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_os = "freebsd")]
lazy_static! {
    pub static ref MATCH_OS: Regex =
        Regex::new(r"(?i:(?:\b|_)freebsd(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_os = "openbsd")]
lazy_static! {
    pub static ref MATCH_OS: Regex =
        Regex::new(r"(?i:(?:\b|_)openbsd(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_os = "macos")]
lazy_static! {
    pub static ref MATCH_OS: Regex =
        Regex::new(r"(?i:(?:\b|_)(?:darwin|macos)(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_os = "windows")]
lazy_static! {
    pub static ref MATCH_OS: Regex =
        Regex::new(r"(?i:(?:\b|_)windows(?:\b|_))").expect("error parsing regex");
}

// =================================================================================
// MATCH_ARCH

#[cfg(target_arch = "aarch64")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)aarch64(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "arm")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)arm(?:v[0-9]+|64)?(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "mips")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)mips(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "mips64")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)mips64(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "powerpc")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)(?:powerpc32(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "powerpc64")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)(?:powerpc64|ppc64(?:le|be)?)?)(?:\b|_))")
            .expect("error parsing regex");
}

#[cfg(target_arch = "riscv")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)riscv(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "s390x")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)s390x(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "sparc")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)sparc(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "sparc64")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)sparc(?:64)?(?:\b|_))").expect("error parsing regex");
}

#[cfg(target_arch = "x86")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)(?:x86|386)(?:\b|_(?!64)))").expect("error parsing regex");
}

#[cfg(target_arch = "x86_64")]
lazy_static! {
    pub static ref MATCH_ARCH: Regex =
        Regex::new(r"(?i:(?:\b|_)(?:x86_64|amd64)(?:\b|_))").expect("error parsing regex");
        // Regex::new(r"(?i:(?:\b|_)(?:x86|386|x86_64|amd64)(?:\b|_))").expect("error parsing regex");
}
