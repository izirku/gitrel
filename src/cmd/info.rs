use crate::cli::InfoArgs;
use crate::domain::github::GitHub;
use crate::domain::util;
use anyhow::Result;

pub async fn info(args: InfoArgs) -> Result<()> {
    let (user, repo, requested_ver) = util::parse_gh_repo_spec(&args.repo_spec)?;
    let gh = GitHub::create(args.token.as_ref());

    let release = gh
        .find_new(
            &user,
            &repo,
            &requested_ver,
            args.asset_glob.as_deref(),
            args.asset_re.as_deref(),
        )
        .await;

    match release {
        Ok(release) => {
            println!("\n         tag: {}", &release.tag_name);
            println!("published at: {}", &release.published_at);
            println!("   file name: {}", &release.assets[0].name);
            println!(
                "        size: {}",
                bytesize::to_string(release.assets[0].size, false)
            );
            println!("   downloads: {}", release.assets[0].download_count);

            Ok(())
        }
        Err(e) => {
            use crate::domain::error::GithubError;
            match e {
                GithubError::AnyHow(e) => Err(e),
                e => {
                    eprint!("\nreason: {}\n\n", e);
                    Ok(())
                }
            }
        }
    }
}
