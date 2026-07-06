use anyhow::Error as AnyhowError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FetchError {
    #[error("{0}")]
    RegistryStatus(String),
    #[error(transparent)]
    Unexpected(#[from] AnyhowError),
}
