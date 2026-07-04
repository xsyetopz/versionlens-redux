use super::{parse_yarnrc_npm_auth_entries_with_env, parse_yarnrc_npm_registry_entries_with_env};

#[test]
fn parses_yarnrc_npm_registries_and_scope_registries() {
    let env = vec![(
        "REGISTRY_HOST".to_owned(),
        "registry.example.test".to_owned(),
    )];
    let entries = parse_yarnrc_npm_registry_entries_with_env(
        r#"
npmRegistryServer: "https://${REGISTRY_HOST}"
npmScopes:
  scope:
    npmRegistryServer: "https://scope.example.test/npm"
  '@quoted':
    npmRegistryServer: "https://quoted.example.test"
"#,
        &env,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].scope, None);
    assert_eq!(entries[0].url, "https://registry.example.test");
    assert_eq!(entries[1].scope.as_deref(), Some("@scope"));
    assert_eq!(entries[1].url, "https://scope.example.test/npm");
    assert_eq!(entries[2].scope.as_deref(), Some("@quoted"));
}

#[test]
fn parses_yarnrc_npm_auth_tokens() {
    let env = vec![("TOKEN".to_owned(), "secret-token".to_owned())];
    let entries = parse_yarnrc_npm_auth_entries_with_env(
        r#"
npmRegistryServer: "https://registry.example.test"
npmAuthToken: "${TOKEN}"
npmScopes:
  scope:
    npmRegistryServer: "https://scope.example.test/npm/"
    npmAuthToken: scoped-token
npmRegistries:
  "https://other.example.test":
    npmAuthToken: other-token
"#,
        &env,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Bearer secret-token");
    assert_eq!(entries[1].registry, "//scope.example.test/npm");
    assert_eq!(entries[1].header_value, "Bearer scoped-token");
    assert_eq!(entries[2].registry, "//other.example.test");
    assert_eq!(entries[2].header_value, "Bearer other-token");
}

#[test]
fn parses_yarnrc_npm_auth_ident_as_basic_auth() {
    let env = vec![("IDENT".to_owned(), "user:pass".to_owned())];
    let entries = parse_yarnrc_npm_auth_entries_with_env(
        r#"
npmRegistryServer: "https://registry.example.test"
npmAuthIdent: "${IDENT}"
npmRegistries:
  "https://other.example.test":
    npmAuthIdent: other:pass
"#,
        &env,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
    assert_eq!(entries[1].registry, "//other.example.test");
    assert_eq!(entries[1].header_value, "Basic b3RoZXI6cGFzcw==");
}
