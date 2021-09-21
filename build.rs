fn main() {
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows",)))]
    {
        eprintln!("trying to build on an unsupported target");
        std::process::exit(1);
    }
}
