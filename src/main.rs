// use gitrel::foundation::config::{ensure_gitignore, get_or_create_cofig_file};
use anyhow::{Context, Result};
use clap::{App, crate_name, crate_version, load_yaml };
use directories::ProjectDirs;
use gitrel::app;
use gitrel::app::list::list_requested;
use gitrel::business::data::conf;
use gitrel::foundation::file;
use std::fs;

fn main() -> Result<()> {
    let proj_dirs = ProjectDirs::from("com.github", "izirku", crate_name!()).unwrap();

    let cfg_dir = proj_dirs.config_dir();
    fs::create_dir_all(cfg_dir)
        .with_context(|| format!("unable to create config dir: {:?}", cfg_dir))?;

    let config_file = cfg_dir.join("config.toml");
    let gh_token_file = cfg_dir.join("github_token.plain");
    let gh_ignore_file = cfg_dir.join(".gitignore");
    let pkg_requested_file = cfg_dir.join("requested.toml");

    let config = conf::get_or_create_cofig_file(&config_file)?;
    file::ensure_gitignore(&gh_ignore_file)?;

    let cli_yaml = load_yaml!("cli.yaml");
    let matches = App::from(cli_yaml).version(crate_version!()).get_matches();

    let token = match matches.value_of("token") {
        Some(token) => Some(token.to_owned()),
        None => file::gh_token_from_file(&gh_token_file),
    };

    dbg!(config);
    dbg!(&token);

    // if let Some(matches) = matches.subcommand_matches("info") {
    //     let _res = tokio::runtime::Builder::new_current_thread()
    //         .enable_all()
    //         .build()
    //         .unwrap()
    //         .block_on(app::info::info(matches.value_of("repo").unwrap(), token));
    // }

    match matches.subcommand() {
        Some(("info", sub_m)) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(app::info::info(
                sub_m.value_of("repo").unwrap(),
                token.as_ref(),
            )),
        Some(("list", _sub_m)) => list_requested(&pkg_requested_file),
        Some(("upgrade", _sub_m)) => tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(app::update::update_requested(
                &pkg_requested_file,
                token.as_ref(),
            )),
        _ => Ok(()),
    }
}
