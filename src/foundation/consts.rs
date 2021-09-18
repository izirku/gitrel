#[cfg(target_os = "linux")]
pub const OS: &str = "linux";

#[cfg(target_os = "windows")]
pub const OS: &str = "windows";

#[cfg(target_os = "macos")]
pub const OS: &str = "macos";

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "windows"),
    not(target_os = "macos")
))]
pub const OS: &str = "unsupported";

#[cfg(target_arch = "x86_64")]
pub const ARCH: &str = "x86_64";

#[cfg(not(target_arch = "x86_64"))]
pub const ARCH: &str = "unsupported";

// TODO: worry about these later
// #[cfg(target_arch = "aarch64")]
// pub const ARCH: &str = "aarch64";
//
// #[cfg(target_arch = "arm")]
// pub const ARCH: &str = "arm";
