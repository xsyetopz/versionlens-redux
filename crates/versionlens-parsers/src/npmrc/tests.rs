use super::{
    parse_npm_env_http_config, parse_npm_env_registry_entries, parse_npmrc_auth_entries_with_env,
    parse_npmrc_client_cert_entries_with_env, parse_npmrc_http_config_with_env,
    parse_npmrc_registry_entries, parse_npmrc_registry_entries_with_env,
};

#[test]
fn parses_npmrc_default_and_scoped_registries() {
    let entries = parse_npmrc_registry_entries(
        r#"
# comment
registry=https://registry.example.test/
@scope:registry = "https://scope.example.test/npm"
; comment
//registry.example.test/:_authToken=${NPM_TOKEN}
"#,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].scope, None);
    assert_eq!(entries[0].url, "https://registry.example.test/");
    assert_eq!(entries[1].scope.as_deref(), Some("@scope"));
    assert_eq!(entries[1].url, "https://scope.example.test/npm");
}

#[test]
fn expands_npmrc_registry_environment_variables() {
    let env = vec![
        (
            "REGISTRY_URL".to_owned(),
            "https://env.example.test".to_owned(),
        ),
        (
            "SCOPE_REGISTRY".to_owned(),
            "https://scope-env.example.test/npm".to_owned(),
        ),
    ];
    let entries = parse_npmrc_registry_entries_with_env(
        "registry=${REGISTRY_URL}\n@scope:registry=${SCOPE_REGISTRY}\n",
        &env,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].url, "https://env.example.test");
    assert_eq!(entries[1].scope.as_deref(), Some("@scope"));
    assert_eq!(entries[1].url, "https://scope-env.example.test/npm");
}

#[test]
fn parses_npmrc_auth_tokens_with_environment_variables() {
    let env = vec![("NPM_TOKEN".to_owned(), "secret-token".to_owned())];
    let entries = parse_npmrc_auth_entries_with_env(
        r#"
//registry.example.test/:_authToken=${NPM_TOKEN}
//scope.example.test/npm/:_authToken="literal-token"
_authToken=ignored
//empty.example.test/:_authToken=
"#,
        &env,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Bearer secret-token");
    assert_eq!(entries[1].registry, "//scope.example.test/npm");
    assert_eq!(entries[1].header_value, "Bearer literal-token");
}

#[test]
fn parses_npmrc_basic_auth_entries() {
    let env = vec![("BASIC_TOKEN".to_owned(), "dXNlcjpwYXNz".to_owned())];
    let entries = parse_npmrc_auth_entries_with_env(
        r#"
//registry.example.test/:_auth=${BASIC_TOKEN}
//scope.example.test/npm/:_auth="literal-basic"
"#,
        &env,
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
    assert_eq!(entries[1].registry, "//scope.example.test/npm");
    assert_eq!(entries[1].header_value, "Basic literal-basic");
}

#[test]
fn parses_npmrc_username_password_auth_entries() {
    let entries = parse_npmrc_auth_entries_with_env(
        r#"
//registry.example.test/:username=user
//registry.example.test/:_password=cGFzcw==
//scope.example.test/npm/:username=${NPM_USER}
//scope.example.test/npm/:_password=${NPM_PASSWORD}
//missing-password.example.test/:username=user
//missing-username.example.test/:_password=cGFzcw==
"#,
        &[
            ("NPM_USER".to_owned(), "scoped".to_owned()),
            ("NPM_PASSWORD".to_owned(), "c2VjcmV0".to_owned()),
        ],
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
    assert_eq!(entries[1].registry, "//scope.example.test/npm");
    assert_eq!(entries[1].header_value, "Basic c2NvcGVkOnNlY3JldA==");
}

#[test]
fn parses_npmrc_http_config_with_environment_variables() {
    let env = vec![
        (
            "HTTPS_PROXY".to_owned(),
            "http://proxy.example.test:8080".to_owned(),
        ),
        ("NPM_CA".to_owned(), "/tmp/npm-ca.pem".to_owned()),
    ];
    let config = parse_npmrc_http_config_with_env(
        r#"
strict-ssl=false
proxy=http://ignored.example.test:8080
https-proxy=${HTTPS_PROXY}
cafile=${NPM_CA}
"#,
        &env,
    );

    assert_eq!(config.strict_ssl, Some(false));
    assert_eq!(
        config.proxy.as_deref(),
        Some("http://proxy.example.test:8080")
    );
    assert_eq!(config.ca_file.as_deref(), Some("/tmp/npm-ca.pem"));
}

#[test]
fn npmrc_http_config_prefers_https_proxy_over_proxy_regardless_order() {
    let config = parse_npmrc_http_config_with_env(
        r#"
https-proxy=http://https-proxy.example.test:8080
proxy=http://plain-proxy.example.test:8080
"#,
        &[],
    );

    assert_eq!(
        config.proxy.as_deref(),
        Some("http://https-proxy.example.test:8080")
    );
}

#[test]
fn parses_npm_environment_registry_entries_and_http_config() {
    let env = vec![
        (
            "NPM_CONFIG_REGISTRY".to_owned(),
            "https://registry.example.test/".to_owned(),
        ),
        (
            "npm_config_@scope:registry".to_owned(),
            "https://scope.example.test/npm".to_owned(),
        ),
        ("npm_config_strict_ssl".to_owned(), "false".to_owned()),
        (
            "npm_config_https_proxy".to_owned(),
            "http://proxy.example.test:8080".to_owned(),
        ),
        ("NPM_CONFIG_CAFILE".to_owned(), "/tmp/env-ca.pem".to_owned()),
    ];

    let registries = parse_npm_env_registry_entries(&env);
    let http = parse_npm_env_http_config(&env);

    assert_eq!(registries.len(), 2);
    assert_eq!(registries[0].scope, None);
    assert_eq!(registries[0].url, "https://registry.example.test/");
    assert_eq!(registries[1].scope.as_deref(), Some("@scope"));
    assert_eq!(registries[1].url, "https://scope.example.test/npm");
    assert_eq!(http.strict_ssl, Some(false));
    assert_eq!(
        http.proxy.as_deref(),
        Some("http://proxy.example.test:8080")
    );
    assert_eq!(http.ca_file.as_deref(), Some("/tmp/env-ca.pem"));
}

#[test]
fn npm_environment_http_config_prefers_https_proxy_over_proxy_regardless_order() {
    let env = vec![
        (
            "NPM_CONFIG_HTTPS_PROXY".to_owned(),
            "http://https-proxy.example.test:8080".to_owned(),
        ),
        (
            "NPM_CONFIG_PROXY".to_owned(),
            "http://plain-proxy.example.test:8080".to_owned(),
        ),
    ];

    let http = parse_npm_env_http_config(&env);

    assert_eq!(
        http.proxy.as_deref(),
        Some("http://https-proxy.example.test:8080")
    );
}

#[test]
fn parses_npmrc_direct_tls_pem_options() {
    let config = parse_npmrc_http_config_with_env(
        "ca=${NPM_CA}\ncert=${NPM_CERT}\nkey=${NPM_KEY}\n",
        &[
            (
                "NPM_CA".to_owned(),
                "-----BEGIN CERTIFICATE-----\nca\n-----END CERTIFICATE-----".to_owned(),
            ),
            (
                "NPM_CERT".to_owned(),
                "-----BEGIN CERTIFICATE-----\ncert\n-----END CERTIFICATE-----".to_owned(),
            ),
            (
                "NPM_KEY".to_owned(),
                "-----BEGIN PRIVATE KEY-----\nkey\n-----END PRIVATE KEY-----".to_owned(),
            ),
        ],
    );

    assert_eq!(
        config.ca.as_deref(),
        Some("-----BEGIN CERTIFICATE-----\nca\n-----END CERTIFICATE-----")
    );
    assert_eq!(
        config.cert.as_deref(),
        Some("-----BEGIN CERTIFICATE-----\ncert\n-----END CERTIFICATE-----")
    );
    assert_eq!(
        config.key.as_deref(),
        Some("-----BEGIN PRIVATE KEY-----\nkey\n-----END PRIVATE KEY-----")
    );
}

#[test]
fn parses_npm_environment_direct_tls_pem_options() {
    let http = parse_npm_env_http_config(&[
        ("NPM_CONFIG_CA".to_owned(), "env-ca".to_owned()),
        ("NPM_CONFIG_CERT".to_owned(), "env-cert".to_owned()),
        ("NPM_CONFIG_KEY".to_owned(), "env-key".to_owned()),
    ]);

    assert_eq!(http.ca.as_deref(), Some("env-ca"));
    assert_eq!(http.cert.as_deref(), Some("env-cert"));
    assert_eq!(http.key.as_deref(), Some("env-key"));
}

#[test]
fn parses_npmrc_quoted_tls_pem_options_with_json_escapes() {
    let config = parse_npmrc_http_config_with_env(
        r#"ca="-----BEGIN CERTIFICATE-----\nca\n-----END CERTIFICATE-----"
cert="-----BEGIN CERTIFICATE-----\ncert\n-----END CERTIFICATE-----"
key="-----BEGIN PRIVATE KEY-----\nkey\n-----END PRIVATE KEY-----"
"#,
        &[],
    );

    assert_eq!(
        config.ca.as_deref(),
        Some(
            "-----BEGIN CERTIFICATE-----
ca
-----END CERTIFICATE-----"
        )
    );
    assert_eq!(
        config.cert.as_deref(),
        Some(
            "-----BEGIN CERTIFICATE-----
cert
-----END CERTIFICATE-----"
        )
    );
    assert_eq!(
        config.key.as_deref(),
        Some(
            "-----BEGIN PRIVATE KEY-----
key
-----END PRIVATE KEY-----"
        )
    );
}

#[test]
fn parses_npmrc_ca_array_as_multiple_pem_entries() {
    let config = parse_npmrc_http_config_with_env(
        r#"ca[]=first-ca
ca[]=second-ca
"#,
        &[],
    );

    assert_eq!(
        config.ca.as_deref(),
        Some(
            "first-ca
second-ca"
        )
    );
}

#[test]
fn parses_npmrc_registry_scoped_client_cert_files() {
    let entries = parse_npmrc_client_cert_entries_with_env(
        "//registry.example.test/:certfile=${CERT_FILE}
//registry.example.test/:keyfile=${KEY_FILE}
//other.example.test/:certfile=/tmp/other-cert.pem
",
        &[
            ("CERT_FILE".to_owned(), "/tmp/client-cert.pem".to_owned()),
            ("KEY_FILE".to_owned(), "/tmp/client-key.pem".to_owned()),
        ],
    );

    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(
        entries[0].cert_file.as_deref(),
        Some("/tmp/client-cert.pem")
    );
    assert_eq!(entries[0].key_file.as_deref(), Some("/tmp/client-key.pem"));
    assert_eq!(entries[1].registry, "//other.example.test");
    assert_eq!(entries[1].cert_file.as_deref(), Some("/tmp/other-cert.pem"));
    assert_eq!(entries[1].key_file, None);
}

#[test]
fn parses_npmrc_fetch_timeout_http_config() {
    let config = parse_npmrc_http_config_with_env(
        "fetch-timeout=${NPM_FETCH_TIMEOUT}
",
        &[("NPM_FETCH_TIMEOUT".to_owned(), "45000".to_owned())],
    );

    assert_eq!(config.timeout_ms, Some(45_000));
}

#[test]
fn parses_npm_environment_fetch_timeout_http_config() {
    let http =
        parse_npm_env_http_config(&[("NPM_CONFIG_FETCH_TIMEOUT".to_owned(), "60000".to_owned())]);

    assert_eq!(http.timeout_ms, Some(60_000));
}

#[test]
fn parses_npmrc_zero_fetch_timeout_for_registry_fetch_fallback_parity() {
    let config = parse_npmrc_http_config_with_env(
        "fetch-timeout=0
",
        &[],
    );

    assert_eq!(config.timeout_ms, Some(0));
}

#[test]
fn parses_npmrc_noproxy_http_config() {
    let config = parse_npmrc_http_config_with_env(
        "noproxy=registry.example.test, .internal.test
",
        &[],
    );

    assert_eq!(
        config.no_proxy.as_deref(),
        Some("registry.example.test, .internal.test")
    );
}

#[test]
fn parses_npm_environment_noproxy_and_generic_proxy_fallback() {
    let http = parse_npm_env_http_config(&[
        (
            "HTTPS_PROXY".to_owned(),
            "http://generic-proxy.example.test:8080".to_owned(),
        ),
        ("NO_PROXY".to_owned(), "registry.example.test".to_owned()),
    ]);

    assert_eq!(http.proxy, None);
    assert_eq!(
        http.generic_proxy.https.as_deref(),
        Some("http://generic-proxy.example.test:8080")
    );
    assert_eq!(http.no_proxy.as_deref(), Some("registry.example.test"));
}

#[test]
fn npmrc_proxy_false_disables_configured_proxy() {
    let config = parse_npmrc_http_config_with_env(
        "proxy=false
https-proxy=http://ignored.example.test:8080
",
        &[],
    );

    assert!(config.proxy.is_none());
    assert!(config.proxy_disabled);
}
