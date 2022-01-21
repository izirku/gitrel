use std::collections::HashSet;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;

use lazy_static::lazy_static;

fn main() {
    let tgt_os = platforms::target::TARGET_OS.as_str();
    let tgt_arch = platforms::target::TARGET_ARCH.as_str();
    // let curr_abi = platforms::target::TARGET_ENV.unwrap().as_str();

    let mut exclude_set: HashSet<&'static str> = ALL_EXCLUDES.iter().copied().collect();

    exclude_set.remove(tgt_os);
    if tgt_os == "macos" {
        exclude_set.remove("apple");
        exclude_set.remove("darwin");
    }
    if tgt_os == "windows" {
        exclude_set.remove("win64");
    }

    exclude_set.remove(tgt_arch);
    if tgt_arch == "x86_64" {
        exclude_set.remove("amd64");
    }
    if tgt_arch == "x86" {
        exclude_set.remove("386");
        exclude_set.remove("i586");
        exclude_set.remove("i686");
        exclude_set.remove("32-bit");
    }

    if let Some(abi) = platforms::target::TARGET_ENV {
        exclude_set.remove(abi.as_str());
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut msg = String::with_capacity(exclude_set.len() * 12);
    msg.write_str("lazy_static! { static ref EXCLUDE_SET: HashSet<&'static str> = vec![")
        .unwrap();
    for term in exclude_set.iter().copied() {
        msg.write_str(format!("\"{}\",", term).as_str()).unwrap();
    }
    msg.write_str("].iter().copied().collect();}\n").unwrap();

    fs::write(&dest_path, msg).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
}

// ========================================================================
// combined values from:
//   1. https://gist.github.com/asukakenji/f15ba7e588ac42795f421b48b8aede63
//   2. rustup target list
lazy_static! {
    pub static ref ALL_EXCLUDES: Vec<&'static str> = vec![
        // ===============================================
        // also good to exclude
        "source",
        "src",
        "vsix",
        "win64",
        "txt",
        // ===============================================
        // OS
        "aix",
        "android",
        "apple",
        "darwin",
        "dragonfly",
        "freebsd",
        "fuchsia",
        "hurd",
        "illumos",
        "ios",
        "js",
        "linux",
        "macos",
        "nacl",
        "netbsd",
        "openbsd",
        "plan9",
        "redox",
        "solaris",
        "sun",
        "windows",
        "zos",
        // ===============================================
        // ARCH
        "32-bit",
        "386",
        "aarch64",
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
        "i586",
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
        "x86_64",
        // ===============================================
        // ABI
        "androideabi",
        "eabi",
        "eabihf",
        "gnu",
        "gnuabi64",
        "gnuabihf",
        "gnueabi64",
        "gnueabihf",
        "gnux32",
        "msvc",
        "musl",
        "muslabi64",
        "musleabi",
        "musleabihf",
        "sgx",
        "uclibc",
    ];
}
