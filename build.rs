fn main() {
    #[cfg(not(any(
        target_os = "linux",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "windows",
    )))]
    {
        println!("trying to build on an unsupported target");
        std::process::exit(1);
    }
}
