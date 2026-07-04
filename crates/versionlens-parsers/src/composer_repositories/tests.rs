use super::{parse_composer_repositories, parse_composer_repository_urls};

#[test]
fn parses_composer_repository_urls_from_arrays_and_objects() {
    let array_urls = parse_composer_repository_urls(
        r#"{
  "repositories": [
    {"type":"composer","url":"https://repo.example.test"},
    {"type":"vcs","url":"https://github.com/example/pkg"}
  ]
}"#,
    );
    let object_urls = parse_composer_repository_urls(
        r#"{
  "repositories": {
    "private": {"type":"composer","url":"https://private.example.test"},
    "disabled": false
  }
}"#,
    );

    assert_eq!(array_urls, vec!["https://repo.example.test"]);
    assert_eq!(object_urls, vec!["https://private.example.test"]);
}

#[test]
fn parses_composer_repository_package_filters() {
    let repositories = parse_composer_repositories(
        r#"{
  "repositories": [
    {
      "type": "composer",
      "url": "https://private.example.test",
      "only": ["acme/*", "exact/package"],
      "exclude": ["acme/blocked"]
    }
  ]
}"#,
    );

    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0].url, "https://private.example.test");
    assert_eq!(repositories[0].only, vec!["acme/*", "exact/package"]);
    assert_eq!(repositories[0].exclude, vec!["acme/blocked"]);
}

#[test]
fn parses_disabled_packagist_repository() {
    assert!(super::parse_composer_packagist_disabled(
        r#"{"repositories":{"packagist.org": false}}"#
    ));
    assert!(super::parse_composer_packagist_disabled(
        r#"{"repositories":[{"packagist.org": false}]}"#
    ));
    assert!(!super::parse_composer_packagist_disabled(
        r#"{"repositories":{"packagist.org": true}}"#
    ));
}

#[test]
fn parses_composer_auth_entries() {
    let entries = super::parse_composer_auth_entries(
        r#"{
  "http-basic": {
    "repo.example.test": {"username":"user","password":"pass"},
    "https://scoped.example.test/path/": {"username":"scoped","password":"secret"}
  },
  "bearer": {
    "bearer.example.test": "token"
  }
}"#,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].registry, "repo.example.test");
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
    assert_eq!(entries[1].registry, "scoped.example.test/path");
    assert_eq!(entries[1].header_value, "Basic c2NvcGVkOnNlY3JldA==");
    assert_eq!(entries[2].registry, "bearer.example.test");
    assert_eq!(entries[2].header_value, "Bearer token");
}
