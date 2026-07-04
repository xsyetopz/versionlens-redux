use super::{parse_bunfig_npm_auth_entries_with_env, parse_bunfig_npm_registry_entries_with_env};

#[test]
fn parses_bunfig_registry_and_scope_registries() {
    let env = vec![(
        "REGISTRY_HOST".to_owned(),
        "registry.example.test".to_owned(),
    )];
    let entries = parse_bunfig_npm_registry_entries_with_env(
        r#"
[install]
registry = "https://${REGISTRY_HOST}"

[install.scopes]
scope = "https://scope.example.test/npm"
"@quoted" = { url = "https://quoted.example.test", token = "${TOKEN}" }
"#,
        &env,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].scope, None);
    assert_eq!(entries[0].url, "https://registry.example.test");
    assert_eq!(entries[1].scope.as_deref(), Some("@scope"));
    assert_eq!(entries[1].url, "https://scope.example.test/npm");
    assert_eq!(entries[2].scope.as_deref(), Some("@quoted"));
    assert_eq!(entries[2].url, "https://quoted.example.test");
}

#[test]
fn parses_bunfig_scope_auth_tokens() {
    let env = vec![("TOKEN".to_owned(), "secret-token".to_owned())];
    let entries = parse_bunfig_npm_auth_entries_with_env(
        r#"
[install.scopes]
"@scope" = { url = "https://scope.example.test/npm/", token = "${TOKEN}" }
empty = { url = "https://empty.example.test", token = "" }
string = "https://string.example.test"
"#,
        &env,
    );

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].registry, "//scope.example.test/npm");
    assert_eq!(entries[0].header_value, "Bearer secret-token");
}
