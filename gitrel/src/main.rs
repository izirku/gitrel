mod cli;
mod cmd;
mod domain;

use crate::cli::Cli;
use anyhow::Result;
use clap::Parser;
use std::future::Future;

fn main() -> Result<()> {
    let args = Cli::parse();

    // println!("{:#?}", &args);
    // Ok(())

    match args.command {
        // Commands::List => cmd::list(),
        cli::Commands::Install(ref args) => rt_current_thread(cmd::install(args)),

        // Commands::Update { bin_names, token } => {
        //     rt_current_thread(cmd::update(bin_names, token.as_ref()))
        // }

        // Commands::Uninstall { bin_names } => rt_current_thread(cmd::uninstall(bin_names)),
        _ => unimplemented!(),
    }
    // match matches.subcommand() {
    //     Some(("info", sub_m)) => rt_current_thread(cmd::info(sub_m)),
    //     Some(("list", sub_m)) => cmd::list(sub_m),
    //     Some(("install", sub_m)) => rt_current_thread(cmd::install(sub_m)),
    //     Some(("update", sub_m)) => rt_current_thread(cmd::update(sub_m)),
    //     Some(("uninstall", sub_m)) => rt_current_thread(cmd::uninstall(sub_m)),
    //     _ => Ok(()),
    // }
}

// repo,
// token,
// rename_binary,
// strip,
// force,
// archive_contains,
// archive_re,
// entry_contains,
// entry_re,

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
