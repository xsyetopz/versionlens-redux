#[test]
fn reads_go_versions_from_normalized_version_arrays() {
    assert_eq!(
        latest_version_from_response(
            Go,
            "go.uber.org/zap",
            r#"{"versions":["v0.32.3","v0.19.10","v0.26.0","v0.23.0-alpha.3"]}"#,
        ),
        Some("v0.32.3".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Go,
            "github.com/docker/cli",
            r#"["v26.1.3+incompatible","v27.0.0+incompatible"]"#,
        ),
        Some("v27.0.0".to_owned())
    );
}

#[test]
fn reads_go_latest_version_from_proxy_info_object() {
    assert_eq!(
        latest_version_from_response(
            Go,
            "golang.org/x/mod",
            r#"{"Version":"v0.2.0","Time":"2020-01-02T17:33:45Z"}"#,
        ),
        Some("v0.2.0".to_owned())
    );
}

#[test]
fn go_module_proxy_versions_fall_back_to_highest_prerelease_when_no_release_exists() {
    assert_eq!(
        latest_version_from_response(
            Go,
            "example.com/mod",
            r#"{"versions":["v1.0.0-beta.1","v1.0.0-beta.2"]}"#,
        ),
        Some("v1.0.0-beta.2".to_owned())
    );
}

#[test]
fn go_module_proxy_versions_fall_back_to_most_recent_pseudo_version_when_no_release_or_prerelease_exists()
 {
    assert_eq!(
        latest_version_from_response(
            Go,
            "example.com/mod",
            r#"{"versions":["v1.2.4-0.20200101000000-aaaaaaaaaaaa","v1.2.3-0.20240202000000-bbbbbbbbbbbb"]}"#,
        ),
        Some("v1.2.3-0.20240202000000-bbbbbbbbbbbb".to_owned())
    );
}

#[test]
fn go_json_metadata_filters_retracted_and_deprecated_versions_when_available() {
    assert_eq!(
        latest_version_from_response(
            Go,
            "example.com/mod",
            r#"{"versions":[{"Version":"v1.0.0"},{"Version":"v1.1.0","Retracted":["bad release"]},{"Version":"v1.2.0-beta.1","Deprecated":"use example.com/mod/v2"}]}"#,
        ),
        Some("v1.0.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Go,
            "example.com/mod",
            r#"{"versions":[{"Version":"v1.0.0"},{"Version":"v1.1.0","Deprecated":"use example.com/mod/v2"}]}"#,
        ),
        Some("v1.0.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Go,
            "example.com/mod",
            r#"[{"Version":"v1.0.0"},{"Version":"v1.1.0-beta.1","Retracted":[]},{"Version":"v1.2.0-beta.1","Retracted":null}]"#,
            true,
        ),
        Some("v1.0.0".to_owned())
    );
}
