use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpError {
    #[error(transparent)]
    Client(#[from] ureq::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Schema(#[from] serde_json::Error),
    #[error("{0}")]
    SchemaValidation(String),
}

impl HttpError {
    pub fn status_code(&self) -> Option<u16> {
        match self {
            Self::Client(ureq::Error::StatusCode(status)) => Some(*status),
            Self::Client(_) | Self::Io(_) | Self::Schema(_) | Self::SchemaValidation(_) => None,
        }
    }
}
