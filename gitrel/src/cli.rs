use clap::{AppSettings, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// install binaries
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Install(InstallArgs),

    /// update binaries
    Update {
        /// binary name(s)
        bin_names: Vec<String>,

        /// GitHub API token
        #[clap(short, long, env = "GITREL_TOKEN")]
        token: Option<String>,
    },

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

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// GitHub user/repo
    #[clap(value_name = "REPO")]
    pub repo_spec: String,

    /// GitHub API token
    #[clap(short, long, env = "GITREL_TOKEN")]
    pub token: Option<String>,

    /// rename binary before installation
    #[clap(short, long = "rename", value_name = "NEW_NAME")]
    pub rename_binary: Option<String>,

    /// minimize by using `strip`
    #[clap(short, long)]
    pub strip: bool,

    /// force [re]install
    #[clap(short, long)]
    pub force: bool,

    /// asset name contains
    #[clap(short = 'a', long = "asset-contains", value_name = "TEXT")]
    pub asset_contains: Option<String>,

    /// asset name matches RegEx
    #[clap(
        short = 'A',
        long = "asset-regex-match",
        value_name = "REGEX",
        conflicts_with = "asset-contains"
    )]
    pub asset_re: Option<String>,

    /// archive asset's entry name contains
    #[clap(short = 'e', long = "entry-contains", value_name = "TEXT")]
    pub entry_contains: Option<String>,

    /// archive asset's entry name matches RegEx
    #[clap(
        short = 'E',
        long = "entry-regex-match",
        value_name = "REGEX",
        conflicts_with = "entry-contains"
    )]
    pub entry_re: Option<String>,
}
