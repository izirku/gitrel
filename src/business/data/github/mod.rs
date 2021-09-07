use crate::business::client;
use anyhow::Result;
use reqwest::Client;

pub mod model;

// use super::conf::model::PackageRequested;
//
// pub fn find_requested(name: &str, spec: &PackageRequested) {}
pub struct GitHub {
    client: Client,
    per_page: u32,
    page: u32,
}

// struct GitHubBuilder {
//     client: Option<Client>,
//     per_page: Option<u32>,
//     page: Option<u32>,
// }

impl GitHub {
//     pub fn build(token: Option<&String>) -> Result<GitHub> {
//         Ok(Self {
//             client: &client::create(token)?,

//         })
//     }
    pub fn new(token: Option<&String>) -> Result<Self> {
        Ok(Self {
            client: client::create(token)?,
            per_page: 20,
            page: 1,
        })
    }

    pub fn per_page(&mut self, per_page: u32) -> &mut Self {
        self.per_page = per_page;
        self
    }
}
