#[derive(thiserror::Error, Debug)]
pub enum GithubError {
    #[error("repository/release not found")]
    ReleaseNotFound,

    #[error("specified asset not found on GitHub")]
    AssetNotFound,

    #[error("multiple assets matched:\n\n{0}\nconsider using/modifying `--asset-glob` or `--asset-regex` filter")]
    AssetMultipleMatch(String),

    #[error("asset file not found")]
    AssetNoMatch,

    #[error("already up to date")]
    AlreadyUpToDate,

    #[error(transparent)]
    AnyHow(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

#[derive(thiserror::Error, Debug)]
pub enum InstallerError {
    #[error("no binary found matching `{0}` {1} inside an archive `{2}`")]
    EntryNotFound(String, &'static str, String),

    #[error(transparent)]
    AnyHow(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}
