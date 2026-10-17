use serde_json::Error as JsonError;
use std::io::Error as IoError;
use std::time::SystemTime;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP operation deadline exceeded")]
    DeadlineExceeded,
    #[error("HTTP response body exceeded the 64 MiB limit")]
    ResponseTooLarge,
    /// The host reported an exhausted rate limit. Requests to it are held
    /// until `reset`, so this also answers requests that were never sent.
    #[error("rate limited by {host} ({status})")]
    RateLimited {
        host: String,
        status: u16,
        reset: SystemTime,
    },
    #[error(transparent)]
    Client(#[from] reqwest::Error),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    Schema(#[from] JsonError),
    #[error("{0}")]
    SchemaValidation(String),
}

impl HttpError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Client(error) => error.status().map(|status| status.as_u16()),
            Self::RateLimited { status, .. } => Some(*status),
            Self::DeadlineExceeded
            | Self::ResponseTooLarge
            | Self::Io(_)
            | Self::Schema(_)
            | Self::SchemaValidation(_) => None,
        }
    }
}
