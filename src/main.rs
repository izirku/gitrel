use anyhow::Result;
use clap::{crate_version, load_yaml, App};
use gitrel::app;
use gitrel::business::conf::ConfigurationManager;
use std::future::Future;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).version(crate_version!()).get_matches();
    let cm = ConfigurationManager::with_clap_matches(&matches)?;

    match matches.subcommand() {
        Some(("info", sub_m)) => rt_current_thread(app::info::process(&cm, sub_m)),
        Some(("list", _)) => app::list::process(&cm),
        // Some(("update", _sub_m)) => rt_current_thread(app::update::update_requested(&cm)),
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
