use super::{latest_version_from_response, release_versions_from_response};
use versionlens_parsers::Ecosystem;

#[test]
fn picks_latest_stable_composer_version() {
    assert_eq!(
        latest_version_from_response(
            Ecosystem::Composer,
            "phpunit/phpunit",
            r#"{"packages":{"phpunit/phpunit":[{"version":"v1.9"},{"version":"10.0.0"},{"version":"11.0.0-beta.1"},{"version":"10.5.0"}]}}"#,
        ),
        Some("10.5.0".to_owned())
    );
}

#[test]
fn picks_latest_composer_version_from_metadata_keys() {
    assert_eq!(
        latest_version_from_response(
            Ecosystem::Composer,
            "phpunit/phpunit",
            r#"{"packages":{"phpunit/phpunit":{"10.0.0":{},"11.0.0-beta.1":{},"10.5.0":{}}}}"#,
        ),
        Some("10.5.0".to_owned())
    );
}

#[test]
fn normalizes_v_prefixed_packagist_versions() {
    assert_eq!(
        latest_version_from_response(
            Ecosystem::Composer,
            "php-parallel-lint/php-parallel-lint",
            r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v3.1.3"},{"version":"v3.0"}]}}"#,
        ),
        Some("3.1.3".to_owned())
    );
}

#[test]
fn extracts_composer_release_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response(
            Ecosystem::Composer,
            r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v3.1.3"},{"version":"v3.1.2"},{"version":"v3.1.1"},{"version":"v3.1.0"},{"version":"v3.2.0-beta.1"},{"version":"v3.0.1"},{"version":"v3.0"}]}}"#,
        ),
        vec![
            "3.0.0".to_owned(),
            "3.0.1".to_owned(),
            "3.1.0".to_owned(),
            "3.1.1".to_owned(),
            "3.1.2".to_owned(),
            "3.1.3".to_owned(),
            "3.2.0-beta.1".to_owned(),
        ]
    );
}
