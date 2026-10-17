use anyhow::Error as AnyhowError;
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum FetchError {
    #[error("dependency resolution timed out")]
    OperationTimeout,
    #[error("{0}")]
    RegistryStatus(String),
    /// An HTTP error status, with the host that sent it.
    #[error("{message} ({status} {host})")]
    HttpStatus {
        message: &'static str,
        status: u16,
        host: String,
    },
    #[error(transparent)]
    Unexpected(#[from] AnyhowError),
}
