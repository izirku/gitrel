use lazy_static::lazy_static;
use std::collections::HashSet;

fn main() {
    // #[cfg(not(any(
    //     target_os = "linux",
    //     target_os = "openbsd",
    //     target_os = "freebsd",
    //     target_os = "macos",
    //     target_os = "windows",
    // )))]
    // {
    //     println!("trying to build on an unsupported target");
    //     std::process::exit(1);
    // }

    let tgt_os = platforms::target::TARGET_OS.as_str();
    let tgt_arch = platforms::target::TARGET_ARCH.as_str();
    // let curr_abi = platforms::target::TARGET_ENV.unwrap().as_str();

    let mut exclude_os: HashSet<&'static str> = ALL_OS.iter().copied().collect();
    exclude_os.remove(tgt_os);
    if tgt_os == "macos" {
        exclude_os.remove("apple");
        exclude_os.remove("darwin");
    }

    let mut exclude_arch: HashSet<&'static str> = ALL_ARCH.iter().copied().collect();
    exclude_arch.remove(tgt_arch);
    if tgt_arch == "x86_64" {
        exclude_arch.remove("amd64");
    }
    if tgt_arch == "x86" {
        exclude_arch.remove("386");
        exclude_arch.remove("i586");
        exclude_arch.remove("i686");
    }

    let mut exclude_abi: HashSet<&'static str> = ALL_ABI.iter().copied().collect();
    if let Some(abi) = platforms::target::TARGET_ENV {
        exclude_abi.remove(abi.as_str());
    }

    // (x86_64|[a-zA-Z0-9]+)
    // (x86_64|x86\-64|[a-zA-Z0-9]+)
}

// ========================================================================
// combined values from:
//   1. https://gist.github.com/asukakenji/f15ba7e588ac42795f421b48b8aede63
//   2. rustup target list

lazy_static! {
    pub static ref ALL_OS: Vec<&'static str> = vec![
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
    ];
    pub static ref ALL_ARCH: Vec<&'static str> = vec![
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
    ];
    pub static ref ALL_ABI: Vec<&'static str> = vec![
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
