use ureq::{Body, http::Response};

use crate::error::HttpError;

pub(in crate::client) type HttpResponse = Response<Body>;
const MAX_RESPONSE_BODY_BYTES: u64 = 64 * 1024 * 1024;

pub(in crate::client) fn read_response_text(
    mut response: HttpResponse,
) -> Result<String, HttpError> {
    Ok(response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BODY_BYTES)
        .lossy_utf8(true)
        .read_to_string()?)
}

pub(in crate::client) fn read_response_bytes(
    mut response: HttpResponse,
) -> Result<Vec<u8>, HttpError> {
    Ok(response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BODY_BYTES)
        .read_to_vec()?)
}
