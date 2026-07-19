use super::{
    latest_version_from_response, release_versions_from_response,
    release_versions_from_response_for_package,
};
use versionlens_model::Ecosystem::Composer;

#[test]
fn picks_latest_stable_composer_version() {
    assert_eq!(
        latest_version_from_response(
            Composer,
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
            Composer,
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
            Composer,
            "php-parallel-lint/php-parallel-lint",
            r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v3.1.3"},{"version":"v3.0"}]}}"#,
        ),
        Some("3.1.3".to_owned())
    );
}

#[test]
fn extracts_composer_release_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Composer,
            "php-parallel-lint/php-parallel-lint",
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

#[test]
fn extracts_composer_releases_only_for_the_requested_package() {
    let body = r#"{
      "packages": {
        "acme/target": [{"version": "1.0.0"}, {"version": "1.1.0"}],
        "acme/unrelated": [{"version": "9.0.0"}]
      }
    }"#;

    assert_eq!(
        release_versions_from_response_for_package(Composer, "acme/target", body),
        vec!["1.0.0".to_owned(), "1.1.0".to_owned()]
    );
    assert_eq!(
        release_versions_from_response_for_package(Composer, "acme/unrelated", body),
        vec!["9.0.0".to_owned()]
    );
    assert!(release_versions_from_response(Composer, body).is_empty());
}

#[test]
fn extracts_composer_branch_alias_versions_from_metadata() {
    assert_eq!(
        release_versions_from_response_for_package(
            Composer,
            "acme/pkg",
            r#"{
              "packages": {
                "acme/pkg": [
                  {
                    "version": "dev-main",
                    "extra": {
                      "branch-alias": {
                        "dev-main": "1.0.x-dev"
                      }
                    }
                  },
                  {"version": "1.0.0"}
                ]
              }
            }"#,
        ),
        vec!["1.0.0".to_owned(), "1.0.x-dev".to_owned()]
    );
}

#[test]
fn reads_packagist_json_api_package_versions() {
    let body = r#"{
      "package": {
        "name": "monolog/monolog",
        "versions": {
          "3.7.0": {"version": "3.7.0"},
          "3.8.0-beta.1": {"version": "3.8.0-beta.1"},
          "3.6.0": {"version": "3.6.0"}
        }
      }
    }"#;

    assert_eq!(
        latest_version_from_response(Composer, "monolog/monolog", body),
        Some("3.7.0".to_owned())
    );
    assert_eq!(
        release_versions_from_response_for_package(Composer, "monolog/monolog", body),
        vec![
            "3.6.0".to_owned(),
            "3.7.0".to_owned(),
            "3.8.0-beta.1".to_owned(),
        ]
    );
}

#[test]
fn extracts_packagist_json_api_releases_only_when_the_package_matches() {
    let body = r#"{
      "package": {
        "name": "acme/target",
        "versions": {"1.0.0": {}, "1.1.0": {}}
      }
    }"#;

    assert_eq!(
        release_versions_from_response_for_package(Composer, "acme/target", body),
        vec!["1.0.0".to_owned(), "1.1.0".to_owned()]
    );
    assert!(
        release_versions_from_response_for_package(Composer, "acme/unrelated", body).is_empty()
    );
}
