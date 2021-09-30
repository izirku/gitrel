#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("requested resource not found")]
    NotFound,
    #[error("multiple resources found")]
    MultipleResults,
    #[error("unknown method of mathing requested release")]
    UnknownMatchKind,
    #[error("not all requested binaries were installed")]
    PartialInstall,
    #[error("operation failed")]
    OperationFailed,
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
                                   // #[error(transparent)]
                                   // Reqwest(#[from] reqwest::Error),  // source and Display delegate to anyhow::Error
}
