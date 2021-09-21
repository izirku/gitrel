// use std::env;
// use std::fs;
// use std::path::Path;

// const head: &'static str = r###"use lazy_static::lazy_static;
// use regex::Regex;

// lazy_static! {
//     // source: https://regex101.com/r/Ly7O1x/1070
//     pub static ref SEMVER: Regex = Regex::new(r"(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<prerelease>(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+(?P<buildmetadata>[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?").expect("error parsing regex");
//     // These values are those supported by Rust (based on the platforms
//     // crate) and Go (based on
//     // https://gist.github.com/asukakenji/f15ba7e588ac42795f421b48b8aede63).
// "###;

// const tail: &'static str = "\n}";

fn main() {
    #[cfg(not(any(target_family = "unix", target_os = "windows",)))]
    {
        println!("trying to build on an unsupported target");
        std::process::exit(1);
    }

    // let out_dir = env::var_os("OUT_DIR").unwrap();
    // let dest_path = Path::new(&out_dir).join("rx.rs");
    // fs::write(
    //     &dest_path,
    //     "pub fn message() -> &'static str {
    //         \"Hello, World!\"
    //     }
    //     "
    // ).unwrap();
    // println!("cargo:rerun-if-changed=build.rs");

}
