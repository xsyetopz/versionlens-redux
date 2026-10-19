use anyhow::Error as AnyhowError;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use versionlens_model::Dependency;
use versionlens_update::{Suggestion, error, rate_limited};

#[derive(Debug, Error)]
pub(crate) enum FetchError {
    #[error("dependency resolution timed out")]
    OperationTimeout,
    /// The host's rate limit is exhausted until `reset`. Not an error of the
    /// dependency: it is shown as a rate-limited status.
    #[error("rate limited until {}", format_reset_time(*.reset))]
    RateLimited { host: String, reset: SystemTime },
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

impl FetchError {
    /// The suggestion for a dependency whose lookup failed with this error.
    pub(crate) fn into_suggestion(self, dependency: Dependency) -> Suggestion {
        let message = self.to_string();
        match self {
            Self::RateLimited { .. } => rate_limited(dependency, message),
            _ => error(dependency, message),
        }
    }
}

/// The wall-clock time of a reset in UTC, such as `14:05 UTC`, which does not
/// depend on the machine's time zone.
fn format_reset_time(reset: SystemTime) -> String {
    let seconds = reset
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs());
    let minutes = seconds / 60 % (24 * 60);
    format!("{:02}:{:02} UTC", minutes / 60, minutes % 60)
}
