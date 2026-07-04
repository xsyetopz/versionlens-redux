use super::blank_session_config;
use crate::model::config::{NativeHttpConfig, NativeHttpHeader};

#[test]
fn http_proxy_is_trimmed_and_blank_proxy_is_ignored_in_rust() {
    let config = crate::model::config::NativeSessionConfig {
        http: Some(NativeHttpConfig {
            timeout_ms: None,
            strict_ssl: None,
            ca_file: None,
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: Some(" http://localhost:8080 ".to_owned()),
            auth_headers: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.http.proxy.as_deref(), Some("http://localhost:8080"));

    let config = crate::model::config::NativeSessionConfig {
        http: Some(NativeHttpConfig {
            timeout_ms: None,
            strict_ssl: None,
            ca_file: None,
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: Some("   ".to_owned()),
            auth_headers: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert!(config.http.proxy.is_none());
}

#[test]
fn http_ca_file_is_trimmed_and_blank_ca_file_is_ignored_in_rust() {
    let config = crate::model::config::NativeSessionConfig {
        http: Some(NativeHttpConfig {
            timeout_ms: None,
            strict_ssl: None,
            ca_file: Some(" /tmp/versionlens-ca.pem ".to_owned()),
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: None,
            auth_headers: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(
        config.http.ca_file.as_deref(),
        Some("/tmp/versionlens-ca.pem")
    );

    let config = crate::model::config::NativeSessionConfig {
        http: Some(NativeHttpConfig {
            timeout_ms: None,
            strict_ssl: None,
            ca_file: Some("   ".to_owned()),
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: None,
            auth_headers: None,
        }),
        ..blank_session_config()
    }
    .into_core();

    assert!(config.http.ca_file.is_none());
}

#[test]
fn http_header_names_are_trimmed_and_blank_names_are_ignored_in_rust() {
    let config = crate::model::config::NativeSessionConfig {
        http: Some(NativeHttpConfig {
            timeout_ms: None,
            strict_ssl: None,
            ca_file: None,
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            proxy: None,
            auth_headers: Some(vec![
                NativeHttpHeader {
                    name: "   ".to_owned(),
                    value: "ignored".to_owned(),
                    url: None,
                },
                NativeHttpHeader {
                    name: " authorization ".to_owned(),
                    value: " Bearer token ".to_owned(),
                    url: Some(" https://registry.example.test ".to_owned()),
                },
            ]),
        }),
        ..blank_session_config()
    }
    .into_core();

    assert_eq!(config.http.auth_headers.len(), 1);
    assert_eq!(config.http.auth_headers[0].name, "authorization");
    assert_eq!(config.http.auth_headers[0].value, " Bearer token ");
    assert_eq!(
        config.http.auth_headers[0].url.as_deref(),
        Some("https://registry.example.test")
    );
}
