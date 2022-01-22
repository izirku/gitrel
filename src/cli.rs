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
    Update(UpdateArgs),

    /// uninstall binaries
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Uninstall(UninstallArgs),

    /// list installed binaries
    List,

    /// match and show info about an available GitHub repo release
    #[clap(setting(AppSettings::ArgRequiredElseHelp))]
    Info(InfoArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// GitHub user/repo
    #[clap(value_name = "REPO", required = true)]
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
    pub asset_glob: Option<String>,

    /// asset name matches RegEx
    #[clap(
        short = 'A',
        long = "asset-regex-match",
        value_name = "REGEX",
        conflicts_with = "asset-glob"
    )]
    pub asset_re: Option<String>,

    /// archive asset's entry name contains
    #[clap(short = 'e', long = "entry-glob", value_name = "TEXT")]
    pub entry_glob: Option<String>,

    /// archive asset's entry name matches RegEx
    #[clap(
        short = 'E',
        long = "entry-regex-match",
        value_name = "REGEX",
        conflicts_with = "entry-glob"
    )]
    pub entry_re: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpdateArgs {
    /// binary name(s)
    pub bin_names: Vec<String>,

    /// GitHub API token
    #[clap(short, long, env = "GITREL_TOKEN")]
    pub token: Option<String>,
}

#[derive(Args, Debug)]
pub struct UninstallArgs {
    /// binary name(s)
    #[clap(required = true)]
    pub bin_names: Vec<String>,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// GitHub user/repo
    #[clap(value_name = "REPO", required = true)]
    pub repo_spec: String,

    /// GitHub API token
    #[clap(short, long, env = "GITREL_TOKEN")]
    pub token: Option<String>,

    /// rename binary before installation
    #[clap(short, long = "rename", value_name = "NEW_NAME")]
    pub rename_binary: Option<String>,

    /// asset name exactly matches
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
}
