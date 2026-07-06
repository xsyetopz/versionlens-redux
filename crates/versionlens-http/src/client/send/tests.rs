use super::response::read_response_text;

#[test]
fn reads_large_registry_response_bodies() {
    let body = ureq::Body::builder().data("x".repeat(11 * 1024 * 1024));
    let response = ureq::http::Response::builder()
        .status(200)
        .body(body)
        .unwrap();

    let text = read_response_text(response).unwrap();

    assert_eq!(text.len(), 11 * 1024 * 1024);
}
