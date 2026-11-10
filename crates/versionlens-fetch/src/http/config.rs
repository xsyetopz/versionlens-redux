use serde::{Deserialize, Serialize};

/// Settings applied to every HTTP request.
#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpConfig {
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Whether TLS certificates are verified.
    pub strict_ssl: bool,
    /// Proxy URL, which may carry credentials.
    pub proxy: Option<String>,
    /// Path to an extra PEM certificate authority bundle.
    pub ca_file: Option<String>,
    /// Inline PEM certificate authority bundle.
    pub ca: Option<String>,
    /// Path to the PEM client certificate.
    pub cert_file: Option<String>,
    /// Path to the PEM client private key.
    pub key_file: Option<String>,
    /// Inline PEM client certificate.
    pub cert: Option<String>,
    /// Inline PEM client private key.
    pub key: Option<String>,
    /// Headers added to matching requests, such as `Authorization`.
    pub auth_headers: Vec<HttpHeader>,
}

/// A header to send, optionally limited to URLs under a prefix.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    /// Header name.
    pub name: String,
    /// Header value, which `Debug` redacts.
    pub value: String,
    /// URL prefix the header applies to, or `None` for every request.
    pub url: Option<String>,
}

impl HttpConfig {
    /// The default configuration; same as [`standard_http_config`].
    pub fn standard() -> Self {
        standard_http_config()
    }
}

/// A configuration with a 10 second timeout, certificate verification on and nothing else set.
pub fn standard_http_config() -> HttpConfig {
    HttpConfig {
        timeout_ms: 10_000,
        strict_ssl: true,
        proxy: None,
        ca_file: None,
        ca: None,
        cert_file: None,
        key_file: None,
        cert: None,
        key: None,
        auth_headers: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::standard_http_config;

    #[test]
    fn standard_config_has_defaults() {
        let config = standard_http_config();

        assert_eq!(config.timeout_ms, 10_000);
        assert!(config.strict_ssl);
        assert!(config.proxy.is_none());
        assert!(config.ca_file.is_none());
        assert_eq!(config.auth_headers, []);
    }
}

const REDACTED: &str = "[redacted]";

/// Debug output that cannot leak a secret: header values, the private key and
/// proxy credentials print as `[redacted]`.
macro_rules! redacted_debug {
    ($header:ident, header) => {
        impl std::fmt::Debug for $header {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($header))
                    .field("name", &self.name)
                    .field("value", &REDACTED)
                    .field("url", &self.url)
                    .finish()
            }
        }
    };
    ($config:ident, config) => {
        impl std::fmt::Debug for $config {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!($config))
                    .field("timeout_ms", &self.timeout_ms)
                    .field("strict_ssl", &self.strict_ssl)
                    .field("proxy", &self.proxy.as_ref().map(|_| REDACTED))
                    .field("ca_file", &self.ca_file)
                    .field("ca", &self.ca)
                    .field("cert_file", &self.cert_file)
                    .field("key_file", &self.key_file)
                    .field("cert", &self.cert)
                    .field("key", &self.key.as_ref().map(|_| REDACTED))
                    .field("auth_headers", &self.auth_headers)
                    .finish()
            }
        }
    };
}

redacted_debug!(HttpHeader, header);
redacted_debug!(HttpConfig, config);
