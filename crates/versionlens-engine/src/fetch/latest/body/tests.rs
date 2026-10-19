use super::{
    CargoNextPage, cargo_next_page_from_response, cargo_page_from_response, crates_io_relative_url,
    official_crates_io_versions_url,
};

#[test]
fn recognizes_only_official_crates_io_version_endpoints() {
    assert!(official_crates_io_versions_url(
        "https://crates.io/api/v1/crates/serde/versions"
    ));
    assert!(!official_crates_io_versions_url(
        "https://registry.example/api/v1/crates/serde/versions"
    ));
    assert!(!official_crates_io_versions_url(
        "https://crates.io/api/v1/crates/serde/owners/versions"
    ));
}

#[test]
fn follows_only_same_origin_relative_crates_io_pages() {
    let current = "https://crates.io/api/v1/crates/serde/versions?per_page=100";
    assert_eq!(
        crates_io_relative_url(current, "?per_page=100&seek=next"),
        Some("https://crates.io/api/v1/crates/serde/versions?per_page=100&seek=next".to_owned())
    );
    assert_eq!(
        crates_io_relative_url(current, "/api/v1/crates/serde/versions?page=2"),
        Some("https://crates.io/api/v1/crates/serde/versions?page=2".to_owned())
    );
    for next in [
        "https://example.com/page",
        "//example.com/page",
        "versions?page=2",
        "",
    ] {
        assert_eq!(crates_io_relative_url(current, next), None);
    }
}

#[test]
fn rejects_exhausted_or_malformed_crates_io_pagination() {
    assert_eq!(
        cargo_next_page_from_response(
            r#"{"versions":[],"meta":{"next_page":"?per_page=100&seek=next"}}"#
        ),
        Some(CargoNextPage::Next("?per_page=100&seek=next".to_owned()))
    );
    assert_eq!(
        cargo_next_page_from_response(r#"{"versions":[],"meta":{"next_page":null}}"#),
        Some(CargoNextPage::Complete)
    );
    for body in [
        r#"{"versions":[],"meta":{"next_page":7}}"#,
        r#"{"versions":[],"meta":{}}"#,
        r#"{"versions":[],"meta":{"next_page":""}}"#,
        "not json",
    ] {
        assert_eq!(cargo_next_page_from_response(body), None);
    }
}

#[test]
fn accepts_only_full_crates_io_version_pages() {
    let page = cargo_page_from_response(
        r#"{"versions":[{"num":"1.0.0","yanked":false}],"meta":{"next_page":null}}"#,
    )
    .unwrap();
    assert_eq!(page.versions.len(), 1);

    for body in [
        r#"{"crate":{"default_version":"1.0.0"}}"#,
        r#"{"versions":null}"#,
        "not json",
    ] {
        assert!(cargo_page_from_response(body).is_none());
    }
}
