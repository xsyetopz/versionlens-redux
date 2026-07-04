use super::{HttpConfig, HttpConfigInput, HttpHeaderInput};

#[test]
fn config_input_uses_standard_defaults_for_missing_options() {
    let config = HttpConfig::from_input(HttpConfigInput {
        timeout_ms: None,
        strict_ssl: None,
        proxy: None,
        ca_file: None,
        ca: None,
        cert_file: None,
        key_file: None,
        cert: None,
        key: None,
        auth_headers: None,
    });

    assert_eq!(config.timeout_ms, 10_000);
    assert!(config.strict_ssl);
    assert!(config.proxy.is_none());
    assert!(config.ca_file.is_none());
    assert!(config.auth_headers.is_empty());
}

#[test]
fn config_input_trims_proxy_and_ca_file_and_rejects_blank_values() {
    let config = HttpConfig::from_input(HttpConfigInput {
        timeout_ms: Some(250),
        strict_ssl: Some(false),
        proxy: Some(" http://localhost:8080 ".to_owned()),
        ca_file: Some(" /tmp/versionlens-ca.pem ".to_owned()),
        ca: Some(" direct-ca ".to_owned()),
        cert_file: Some(" /tmp/versionlens-client-cert.pem ".to_owned()),
        key_file: Some(" /tmp/versionlens-client-key.pem ".to_owned()),
        cert: Some(" direct-cert ".to_owned()),
        key: Some(" direct-key ".to_owned()),
        auth_headers: None,
    });

    assert_eq!(config.timeout_ms, 250);
    assert!(!config.strict_ssl);
    assert_eq!(config.proxy.as_deref(), Some("http://localhost:8080"));
    assert_eq!(config.ca_file.as_deref(), Some("/tmp/versionlens-ca.pem"));

    let config = HttpConfig::from_input(HttpConfigInput {
        timeout_ms: None,
        strict_ssl: None,
        proxy: Some("   ".to_owned()),
        ca_file: Some(String::new()),
        ca: Some("   ".to_owned()),
        cert_file: Some("   ".to_owned()),
        key_file: Some(String::new()),
        cert: Some("   ".to_owned()),
        key: Some(String::new()),
        auth_headers: None,
    });

    assert!(config.proxy.is_none());
    assert!(config.ca_file.is_none());
}

#[test]
fn config_input_trims_header_names_and_urls_and_rejects_blank_names() {
    let config = HttpConfig::from_input(HttpConfigInput {
        timeout_ms: None,
        strict_ssl: None,
        proxy: None,
        ca_file: None,
        ca: None,
        cert_file: None,
        key_file: None,
        cert: None,
        key: None,
        auth_headers: Some(vec![
            HttpHeaderInput {
                name: "   ".to_owned(),
                value: "ignored".to_owned(),
                url: None,
            },
            HttpHeaderInput {
                name: " authorization ".to_owned(),
                value: " Bearer token ".to_owned(),
                url: Some(" https://registry.example.test ".to_owned()),
            },
            HttpHeaderInput {
                name: "x-global".to_owned(),
                value: " global ".to_owned(),
                url: Some("   ".to_owned()),
            },
        ]),
    });

    assert_eq!(config.auth_headers.len(), 2);
    assert_eq!(config.auth_headers[0].name, "authorization");
    assert_eq!(config.auth_headers[0].value, " Bearer token ");
    assert_eq!(
        config.auth_headers[0].url.as_deref(),
        Some("https://registry.example.test")
    );
    assert_eq!(config.auth_headers[1].name, "x-global");
    assert_eq!(config.auth_headers[1].url, None);
}
