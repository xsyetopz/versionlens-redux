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

#[cfg(test)]
mod tests {
    use ureq::Body;
    use ureq::http::Response;

    use super::read_response_text;

    #[test]
    fn reads_large_registry_response_bodies() {
        let body = Body::builder().data("x".repeat(11 * 1024 * 1024));
        let response = Response::builder().status(200).body(body).unwrap();

        let text = read_response_text(response).unwrap();

        assert_eq!(text.len(), 11 * 1024 * 1024);
    }
}
