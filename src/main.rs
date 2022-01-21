mod cli;
mod cmd;
mod domain;

use std::future::Future;

use anyhow::Result;
use clap::Parser;

use crate::cli::Cli;

fn main() -> Result<()> {
    let args = Cli::parse();

    // println!("{:#?}", &args);
    // Ok(())

    match args.command {
        cli::Commands::Install(args) => rt_current_thread(cmd::install(args)),
        cli::Commands::Update(args) => rt_current_thread(cmd::update(args)),
        cli::Commands::Uninstall(args) => rt_current_thread(cmd::uninstall(args)),
        cli::Commands::List => cmd::list(),
        cli::Commands::Info(args) => rt_current_thread(cmd::info(args)),
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

#[test]
fn verify_app() {
    use clap::IntoApp;
    Cli::into_app().debug_assert()
}
