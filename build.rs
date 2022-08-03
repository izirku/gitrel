use std::collections::HashSet;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use lazy_static::lazy_static;

fn main() {
    let exclude_set: HashSet<&str> = ALL_EXCLUDES.iter().copied().collect();
    let include_set: HashSet<&str> = ALL_INCLUDES.iter().copied().collect();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    panic!("unsupported target Architecture/OS");

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch4")))]
    panic!("unsupported target Architecture/OS");

    #[cfg(all(target_os = "windows", target_arch = "aarch"))]
    panic!("unsupported target Architecture/OS");

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut msg = String::with_capacity((exclude_set.len() + include_set.len()) * 12);

    // generate EXCLUDE_SET
    msg.write_str("lazy_static! { static ref EXCLUDE_SET: HashSet<&'static str> = vec![")
        .unwrap();
    for term in exclude_set.iter() {
        msg.write_str(format!("\"{}\",", term).as_str()).unwrap();
    }
    msg.write_str("].iter().copied().collect();}\n").unwrap();

    // generate INCLUDE_SET
    msg.write_str("lazy_static! { static ref INCLUDE_SET: HashSet<&'static str> = vec![")
        .unwrap();
    for term in include_set.iter() {
        msg.write_str(format!("\"{}\",", term).as_str()).unwrap();
    }
    msg.write_str("].iter().copied().collect();}\n").unwrap();

    fs::write(&dest_path, msg).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

lazy_static! {
    pub static ref ALL_INCLUDES: Vec<&'static str> = vec![
        #[cfg(target_os="linux")]
        "linux",

        #[cfg(target_os="windows")]
        "windows",
        #[cfg(target_os="windows")]
        "win",
        #[cfg(target_os="windows")]
        "exe",

        #[cfg(target_os="macos")]
        "macos",
        #[cfg(target_os="macos")]
        "apple",
        #[cfg(target_os="macos")]
        "darwin",
        #[cfg(target_os="macos")]
        "osx",
    ];

    // combined values from:
    //   1. https://gist.github.com/asukakenji/f15ba7e588ac42795f421b48b8aede63
    //   2. rustup target list
    pub static ref ALL_EXCLUDES: Vec<&'static str> = vec![
        // ===============================================
        // also good to exclude
        "source",
        "src",
        "vsix",

        #[cfg(not(all(target_os="windows", target_arch="x86_64")))]
        "win64",

        #[cfg(not(all(target_os="windows", target_arch="x86")))]
        "win32",

        "txt",
        "deb",
        "rpm",
        "sha256",
        "sha256sum",

        // ===============================================
        // OS
        "aix",
        "android",

        #[cfg(not(target_os="macos"))]
        "apple",
        #[cfg(not(target_os="macos"))]
        "darwin",

        #[cfg(not(target_os="macos"))]
        "osx",

        "dragonfly",
        "freebsd",
        "fuchsia",
        "hurd",
        "illumos",
        "ios",
        "js",

        #[cfg(not(target_os="linux"))]
        "linux",

        #[cfg(not(target_os="macos"))]
        "macos",

        "nacl",
        "netbsd",
        "openbsd",
        "plan9",
        "redox",
        "solaris",
        "sun",

        #[cfg(not(target_os="windows"))]
        "windows",

        #[cfg(not(target_os="windows"))]
        "win",
        "zos",

        // ===============================================
        // ARCH
        #[cfg(not(target_arch="x86"))]
        "32-bit",

        #[cfg(not(target_arch="x86"))]
        "386",

        #[cfg(not(target_arch="aarch64"))]
        "aarch64",

        #[cfg(not(target_arch="x86_64"))]
        "amd64",

        "amd64p32",
        "arm",
        "arm64",
        "arm64be",
        "armbe",
        "armebv7r",
        "armv5te",
        "armv6",
        "armv7",
        "armv7a",
        "armv7r",
        "asmjs",

        #[cfg(not(target_arch="x86"))]
        "i586",

        #[cfg(not(target_arch="x86"))]
        "i686",

        "loong64",
        "mips",
        "mips64",
        "mips64el",
        "mips64le",
        "mips64p32",
        "mips64p32le",
        "mipsel",
        "mipsle",
        "nvptx64",
        "powerpc",
        "powerpc64",
        "powerpc64le",
        "ppc",
        "ppc64",
        "ppc64le",
        "riscv",
        "riscv32i",
        "riscv32imac",
        "riscv32imc",
        "riscv64",
        "riscv64gc",
        "riscv64imac",
        "s390",
        "s390x",
        "sparc",
        "sparc64",
        "sparcv9",
        "thumbv6m",
        "thumbv7em",
        "thumbv7m",
        "thumbv7neon",
        "thumbv8m",
        "wasm",
        "wasm32",

        #[cfg(not(target_arch="x86_64"))]
        "x86_64",

        // ===============================================
        // ABI
        "androideabi",
        "eabi",
        "eabihf",

        #[cfg(not(target_env="gnu"))]
        "gnu",

        "gnuabi64",
        "gnuabihf",
        "gnueabi64",
        "gnueabihf",
        "gnux32",

        #[cfg(not(target_env="msvc"))]
        "msvc",

        #[cfg(not(target_env="musl"))]
        "musl",

        "muslabi64",
        "musleabi",
        "musleabihf",
        "sgx",

        #[cfg(not(target_env="uclibc"))]
        "uclibc",
    ];
}
