mod http;
mod npm;

pub use http::http_status_message_from_code;
pub use npm::npm_error_status_from_response;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryErrorStatus {
    Error(String),
    Invalid(String),
    InvalidWithLatest(String),
    NoMatch(String),
    NotSupported,
}
