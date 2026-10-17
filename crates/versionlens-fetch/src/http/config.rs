use serde::{Deserialize, Serialize};

macro_rules! define_http_config {
    ($name:ident, $(#[$attr:meta])*, $timeout:ty, $strict_ssl:ty, $proxy:ty, $ca_file:ty, $ca:ty, $cert_file:ty, $key_file:ty, $cert:ty, $key:ty, $headers:ty) => {
        $(#[$attr])*
        pub struct $name {
            pub timeout_ms: $timeout,
            pub strict_ssl: $strict_ssl,
            pub proxy: $proxy,
            pub ca_file: $ca_file,
            pub ca: $ca,
            pub cert_file: $cert_file,
            pub key_file: $key_file,
            pub cert: $cert,
            pub key: $key,
            pub auth_headers: $headers,
        }
    };
}

define_http_config!(HttpConfig,
    #[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")],
    u64, bool, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Vec<HttpHeader>);

define_http_config!(HttpConfigInput,
    #[derive(Debug, PartialEq, Eq)],
    Option<u64>, Option<bool>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<Vec<HttpHeaderInput>>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpHeader {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct HttpHeaderInput {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
}

impl HttpHeaderInput {
    pub fn new(name: impl Into<String>, value: impl Into<String>, url: Option<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            url,
        }
    }
}

fn http_header_from_input(input: HttpHeaderInput) -> Option<HttpHeader> {
    let name = input.name.trim();
    if name.is_empty() {
        return None;
    }

    Some(HttpHeader {
        name: name.to_owned(),
        value: input.value,
        url: trim_optional(input.url),
    })
}

impl HttpConfig {
    pub fn standard() -> Self {
        standard_http_config()
    }

    pub fn from_input(input: HttpConfigInput) -> Self {
        let defaults = Self::standard();
        Self {
            timeout_ms: input.timeout_ms.unwrap_or(defaults.timeout_ms),
            strict_ssl: input.strict_ssl.unwrap_or(defaults.strict_ssl),
            proxy: trim_optional(input.proxy),
            ca_file: trim_optional(input.ca_file),
            ca: trim_optional(input.ca),
            cert_file: trim_optional(input.cert_file),
            key_file: trim_optional(input.key_file),
            cert: trim_optional(input.cert),
            key: trim_optional(input.key),
            auth_headers: input
                .auth_headers
                .unwrap_or_default()
                .into_iter()
                .filter_map(http_header_from_input)
                .collect(),
        }
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

impl HttpHeader {
    pub fn from_input(input: HttpHeaderInput) -> Option<Self> {
        let name = input.name.trim();
        if name.is_empty() {
            return None;
        }

        Some(Self {
            name: name.to_owned(),
            value: input.value,
            url: trim_optional(input.url),
        })
    }
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests;

pub fn http_config_from_input(input: HttpConfigInput) -> HttpConfig {
    input.into()
}

impl From<HttpConfigInput> for HttpConfig {
    fn from(input: HttpConfigInput) -> Self {
        Self::from_input(input)
    }
}
