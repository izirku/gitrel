#[derive(thiserror::Error, Debug)]
pub enum ResponseError {
    #[error("requested resource not found")]
    NotFound,
    #[error(transparent)]
    AnyHow(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
                                   // #[error(transparent)]
                                   // Reqwest(#[from] reqwest::Error),  // source and Display delegate to anyhow::Error
}
