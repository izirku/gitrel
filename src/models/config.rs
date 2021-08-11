use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConfigFile {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub bin_dir: Option<String>,
    pub strip: Option<bool>,
}
// pub struct Config<'a> {
//     arch: &'a str,
//     os: &'a str,
//     bin_dir: &'a str,
//     strip: bool,
// }
