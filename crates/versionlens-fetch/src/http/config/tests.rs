use super::standard_http_config;

#[test]
fn standard_config_has_defaults() {
    let config = standard_http_config();

    assert_eq!(config.timeout_ms, 10_000);
    assert!(config.strict_ssl);
    assert!(config.proxy.is_none());
    assert!(config.ca_file.is_none());
    assert!(config.auth_headers.is_empty());
}
