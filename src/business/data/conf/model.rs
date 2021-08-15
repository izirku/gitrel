use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigFile {
    pub arch: Option<String>,
    pub os: Option<String>,
    pub bin_dir: Option<String>,
    pub strip: Option<bool>,
}

pub type PackageReqMap = BTreeMap<String, PackageReq>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PackageReq {
    Simple(String),
    Detailed(PackageReqDetail),
}

impl PackageReq {
    pub fn into_detailed(self, name: String) -> PackageReqDetail {
        use PackageReq::*;
        match self {
            Simple(version) => PackageReqDetail {
                repo: Some(format!("{0}/{0}", name)),
                bin_name: Some(name.to_lowercase()),
                version_requested: Some(version),
                ..Default::default()
            },
            Detailed(detailed) => {
                // let bin_name = detailed
                //     .repo
                //     .as_ref()
                //     .and_then(|repo| repo.split_once('/'))
                //     .and_then(|(_, repo_name)| Some(repo_name.to_lowercase()));
                // PackageReqDetail {
                //     bin_name,
                //     ..detailed
                // }
                PackageReqDetail {
                    bin_name: Some(name.to_lowercase()),
                    ..detailed
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PackageReqDetail {
    pub repo: Option<String>,
    pub bin_name: Option<String>,
    #[serde(rename = "version")]
    pub version_requested: Option<String>,
    pub strip: Option<bool>,
    pub contains: Option<String>,
    pub matches: Option<String>,
}

// #[serde(from = "PackageSpec")]
// pub struct Package {}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct InstalledPackage {}
