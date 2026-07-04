use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct HttpConfigInput {
    pub timeout_ms: Option<u64>,
    pub strict_ssl: Option<bool>,
    pub proxy: Option<String>,
    pub ca_file: Option<String>,
    pub ca: Option<String>,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub auth_headers: Option<Vec<HttpHeaderInput>>,
}

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

impl HttpConfig {
    pub fn standard() -> Self {
        Self {
            timeout_ms: 10_000,
            strict_ssl: true,
            proxy: None,
            ca_file: None,
            ca: None,
            cert_file: None,
            key_file: None,
            cert: None,
            key: None,
            auth_headers: Vec::new(),
        }
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
                .filter_map(HttpHeader::from_input)
                .collect(),
        }
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
