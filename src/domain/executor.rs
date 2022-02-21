use std::ffi::OsString;
use std::path::Path;

use anyhow::Result;

#[cfg(not(target_os = "windows"))]
pub fn exec(bin_path: &Path, cmd: &str) -> Result<()> {
    // using `String` or `str`:
    // let bin_path = bin_path.to_string_lossy();
    // let cmd = cmd.replace(":bin:", &bin_path);
    // let cmd = format!("export f={}; {}", &bin_path, &cmd);

    // using `OsString`:
    let mut to_exec = OsString::from("export f=");
    to_exec.push(bin_path);
    to_exec.push("; ");
    to_exec.push(cmd.replace(":bin:", &bin_path.to_string_lossy()));
    let mut proc = std::process::Command::new("/bin/sh");

    match proc.arg("-c").arg(&to_exec).output() {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::Error::msg(e)),
    }
}
