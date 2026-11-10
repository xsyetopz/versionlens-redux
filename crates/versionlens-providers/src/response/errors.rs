mod npm;

pub use npm::npm_error_status_from_response;

/// The outcome a registry error response maps to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryErrorStatus {
    /// A registry error with its message.
    Error(String),
    /// The package or version is invalid, with the message.
    Invalid(String),
    /// The version is invalid but the registry reports a latest version.
    InvalidWithLatest(String),
    /// No version matches the requirement.
    NoMatch(String),
    /// The response carries no known error.
    NotSupported,
}

const HTTP_STATUS_MESSAGES: &[(u16, &str)] = &[
    (400, "400 bad request"),
    (401, "401 not authorized"),
    (403, "403 forbidden"),
    (404, "not found"),
    (429, "too many requests"),
    (500, "500 internal server error"),
];

/// The message for an HTTP status code, or `None` when the code has none.
pub fn http_status_message_from_code(status: u16) -> Option<&'static str> {
    HTTP_STATUS_MESSAGES
        .iter()
        .find_map(|(code, message)| (*code == status).then_some(*message))
}
