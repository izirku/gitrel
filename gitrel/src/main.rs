mod cmd;
mod domain;

use anyhow::Result;
use clap::{AppSettings, Parser, Subcommand};
use std::future::Future;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// install binaries
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Install {
        /// GitHub user/repo
        repo: String,

        /// GitHub API token
        #[clap(short, long)]
        token: Option<String>,

        /// minimize by using `strip`
        #[clap(short, long)]
        strip: bool,

        /// force [re]install
        #[clap(short, long)]
        force: bool,
    },

    /// update binaries
    Update {
        /// binary name(s)
        bin_names: Vec<String>,

        /// GitHub API token
        #[clap(short, long)]
        token: Option<String>,
    },

    // #[clap(long, short, required = false)]
    /// uninstall binaries
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Uninstall {
        /// binary name(s)
        bin_names: Vec<String>,
    },

    /// list installed binaries
    List,

    /// show info about a GitHub repo available binary releases
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Info {
        /// GitHub user/repo
        repo: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    // println!("{:#?}", &args);
    // Ok(())

    match args.command {
        Commands::List => cmd::list(),

        Commands::Install {
            repo,
            token,
            strip,
            force,
        } => rt_current_thread(cmd::install(repo, token.as_ref(), strip, force)),

        Commands::Update { bin_names, token } => {
            rt_current_thread(cmd::update(bin_names, token.as_ref()))
        }

        Commands::Uninstall { bin_names } => rt_current_thread(cmd::uninstall(bin_names)),
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
