use anyhow::Result;

use crate::cli::InfoArgs;
use crate::domain::github::GitHub;
use crate::domain::util;

pub async fn info(args: InfoArgs) -> Result<()> {
    let (user, repo, requested_ver) = util::parse_gh_repo_spec(&args.repo_spec)?;
    let gh = GitHub::create(args.token.as_ref());

    let release = gh
        .find_new(
            &user,
            &repo,
            &requested_ver,
            args.asset_contains.as_deref(),
            args.asset_re.as_deref(),
        )
        .await;

    match release {
        Ok(Some(release)) => {
            println!("\n         tag: {}", &release.tag_name);
            println!("published at: {}", &release.published_at);
            println!("   file name: {}", &release.assets[0].name);
            println!("        size: {}", bytesize::to_string(release.assets[0].size, false));
            println!("   downloads: {}", release.assets[0].download_count);

            Ok(())
        }
        Ok(None) => {
            println!("not able to find a matching release");
            Ok(())
        }
        Err(e) => Err(anyhow::Error::msg(e).context("operation failed")),
    }
}
