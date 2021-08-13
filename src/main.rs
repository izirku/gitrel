// use gitrel::foundation::config::{ensure_gitignore, get_or_create_cofig_file};
use anyhow::{Context, Result};
use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use directories::ProjectDirs;
use gitrel::app;
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

    let config = conf::get_or_create_cofig_file(&config_file)?;
    file::ensure_gitignore(&gh_ignore_file)?;

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .author(crate_authors!())
        .arg(
            Arg::new("token")
                .about(
                    format!(
                        "GitHub API token
priority: arg -> env -> token file
 [file: {:?}]\n",
                        gh_token_file
                    )
                    .as_str(),
                )
                .next_line_help(true)
                .long("token")
                .short('t')
                .takes_value(true)
                .env("GITREL_TOKEN")
                .hide_env_values(true),
        )
        .subcommand(
            App::new("list").about("list installed apps").arg(
                Arg::new("outdated")
                    .long("outdated")
                    .short('u')
                    .about("show available updates"),
            ),
        )
        .subcommand(
            App::new("info").about("show info about an app").arg(
                Arg::new("repo")
                    .about("GitHub user/repo")
                    .takes_value(true)
                    .required(true),
            ),
        )
        .subcommand(
            App::new("install").about("install apps").arg(
                Arg::new("strip")
                    .short('s')
                    .about("minimize by using `strip`"),
            ),
        )
        .subcommand(App::new("uninstall").about("uninstall apps"))
        .subcommand(App::new("upgrade").alias("update").about("upgrade apps"))
        .get_matches();

    let token = match matches.value_of("token") {
        Some(token) => Some(token.to_owned()),
        None => file::gh_token_from_file(&gh_token_file),
    };

    dbg!(config);
    dbg!(&token);

    if let Some(matches) = matches.subcommand_matches("info") {
        let _res = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(app::info::info(matches.value_of("repo").unwrap(), token));
    }

    Ok(())
}
