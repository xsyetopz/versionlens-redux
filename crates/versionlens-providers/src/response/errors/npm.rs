mod known;
mod response;
mod status;

use known::npm_known_error_status;
use response::npm_response_status;
use status::npm_status_error;

use super::RegistryErrorStatus;

pub fn npm_error_status_from_response(body: &str) -> Option<RegistryErrorStatus> {
    let status = npm_response_status(body)?;
    npm_error_status(&status)
}

fn npm_error_status(status: &str) -> Option<RegistryErrorStatus> {
    let status = trim_status(status)?;

    npm_known_error_status(status).or_else(|| Some(npm_status_error(status)))
}

pub(super) fn trim_status(status: &str) -> Option<&str> {
    let status = status.trim();
    (!status.is_empty()).then_some(status)
}
