mod cmd;
mod domain;

use anyhow::Result;
use clap::{crate_version, load_yaml, App};
use std::future::Future;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).version(crate_version!()).get_matches();

    match matches.subcommand() {
        Some(("info", sub_m)) => rt_current_thread(cmd::info(sub_m)),
        Some(("list", sub_m)) => cmd::list(sub_m),
        Some(("install", sub_m)) => rt_current_thread(cmd::install(sub_m)),
        Some(("update", sub_m)) => rt_current_thread(cmd::update(sub_m)),
        _ => Ok(()),
    }
}

#[inline]
fn rt_current_thread<F: Future>(f: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(f)
}
