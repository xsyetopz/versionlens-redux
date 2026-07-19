use serde_json::Error as JsonError;
use std::io::Error as IoError;
use thiserror::Error;
use ureq::Error as UreqError;
use ureq::Error::StatusCode as UreqStatusCode;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP operation deadline exceeded")]
    DeadlineExceeded,
    #[error(transparent)]
    Client(#[from] UreqError),
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
            Self::Client(UreqStatusCode(status)) => Some(*status),
            Self::DeadlineExceeded
            | Self::Client(_)
            | Self::Io(_)
            | Self::Schema(_)
            | Self::SchemaValidation(_) => None,
        }
    }
}
