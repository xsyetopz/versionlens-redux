use ureq::{Body, http::Response};

use crate::error::HttpError;

pub(super) type HttpResponse = Response<Body>;
const MAX_TEXT_BODY_BYTES: u64 = 64 * 1024 * 1024;

pub(super) fn read_response_text(mut response: HttpResponse) -> Result<String, HttpError> {
    Ok(response
        .body_mut()
        .with_config()
        .limit(MAX_TEXT_BODY_BYTES)
        .lossy_utf8(true)
        .read_to_string()?)
}
