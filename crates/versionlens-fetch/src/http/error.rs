use serde_json::Error as JsonError;
use std::io::Error as IoError;
use std::time::SystemTime;
use thiserror::Error;

/// Why an HTTP request failed.
#[derive(Debug, Error)]
pub enum HttpError {
    /// The overall deadline for the operation passed.
    #[error("HTTP operation deadline exceeded")]
    DeadlineExceeded,
    /// The response body was larger than the 64 MiB limit.
    #[error("HTTP response body exceeded the 64 MiB limit")]
    ResponseTooLarge,
    /// The host reported an exhausted rate limit. Requests to it are held
    /// until `reset`, so this also answers requests that were never sent.
    #[error("rate limited by {host} ({status})")]
    RateLimited {
        /// The rate-limited host.
        host: String,
        /// The HTTP status that reported the limit.
        status: u16,
        /// When requests to the host may resume.
        reset: SystemTime,
    },
    /// The HTTP client failed or the server returned an error status.
    #[error(transparent)]
    Client(#[from] reqwest::Error),
    /// Reading the response or a local file failed.
    #[error(transparent)]
    Io(#[from] IoError),
    /// The response was not valid JSON for the expected shape.
    #[error(transparent)]
    Schema(#[from] JsonError),
    /// The response was well-formed but violated a schema rule, described by the message.
    #[error("{0}")]
    SchemaValidation(String),
}

impl HttpError {
    /// The HTTP status code behind this error, if it has one.
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
