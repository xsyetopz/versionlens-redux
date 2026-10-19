use serde::{Deserialize, Serialize};

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpConfig {
    pub timeout_ms: u64,
    pub strict_ssl: bool,
    pub proxy: Option<String>,
    pub ca_file: Option<String>,
    pub ca: Option<String>,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub auth_headers: Vec<HttpHeader>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
}

impl HttpConfig {
    pub fn standard() -> Self {
        standard_http_config()
    }
}

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
mod tests;

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
