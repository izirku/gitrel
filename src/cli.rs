use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// install binaries
    #[clap(arg_required_else_help = true)]
    Install(InstallArgs),

    /// update binaries
    Update(UpdateArgs),

    /// uninstall binaries
    #[clap(arg_required_else_help = true)]
    Uninstall(UninstallArgs),

    /// list installed binaries
    List(ListArgs),

    /// match and show info about an available GitHub repo release
    #[clap(arg_required_else_help = true)]
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

    /// override installation path
    #[clap(short, long = "install-path", value_name = "INSTALL_PATH")]
    pub path: Option<String>,

    /// minimize by using `strip`
    #[cfg(not(target_os = "windows"))]
    #[clap(short, long)]
    pub strip: bool,

    /// force [re]install
    #[clap(short, long)]
    pub force: bool,

    /// match asset name using glob pattern
    #[clap(short = 'a', long = "asset-glob", value_name = "TEXT")]
    pub asset_glob: Option<String>,

    /// match asset name using RegEx pattern
    #[clap(
        short = 'A',
        long = "asset-regex",
        value_name = "REGEX",
        conflicts_with = "asset-glob"
    )]
    pub asset_re: Option<String>,

    /// match archived asset entry name using glob pattern
    #[clap(short = 'e', long = "entry-glob", value_name = "TEXT")]
    pub entry_glob: Option<String>,

    /// match archived asset entry name using RegEx pattern
    #[clap(
        short = 'E',
        long = "entry-regex",
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
pub struct ListArgs {
    /// display more information (+show installation path)
    #[clap(short, long)]
    pub wide: bool,
}

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// GitHub user/repo
    #[clap(value_name = "REPO", required = true)]
    pub repo_spec: String,

    /// GitHub API token
    #[clap(short, long, env = "GITREL_TOKEN")]
    pub token: Option<String>,

    /// match asset name using glob pattern
    #[clap(short = 'a', long = "asset-glob", value_name = "TEXT")]
    pub asset_glob: Option<String>,

    /// match asset name using RegEx pattern
    #[clap(
        short = 'A',
        long = "asset-regex",
        value_name = "REGEX",
        conflicts_with = "asset-glob"
    )]
    pub asset_re: Option<String>,
}
