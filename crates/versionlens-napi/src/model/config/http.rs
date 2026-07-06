use napi_derive::napi;
use versionlens_http::{HttpConfigInput, HttpHeaderInput};

#[napi(object)]
pub struct NativeHttpConfig {
    pub timeout_ms: Option<u32>,
    pub strict_ssl: Option<bool>,
    pub proxy: Option<String>,
    pub ca_file: Option<String>,
    pub ca: Option<String>,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub auth_headers: Option<Vec<NativeHttpHeader>>,
}

#[napi(object)]
pub struct NativeHttpHeader {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
}

impl NativeHttpConfig {
    pub(in crate::model::config) fn into_input(self) -> HttpConfigInput {
        HttpConfigInput {
            timeout_ms: self.timeout_ms.map(u64::from),
            strict_ssl: self.strict_ssl,
            proxy: self.proxy,
            ca_file: self.ca_file,
            ca: self.ca,
            cert_file: self.cert_file,
            key_file: self.key_file,
            cert: self.cert,
            key: self.key,
            auth_headers: self.auth_headers.map(|headers| {
                headers
                    .into_iter()
                    .map(http_header_input_from_native)
                    .collect()
            }),
        }
    }
}

fn http_header_input_from_native(header: NativeHttpHeader) -> HttpHeaderInput {
    HttpHeaderInput {
        name: header.name,
        value: header.value,
        url: header.url,
    }
}

impl From<NativeHttpConfig> for HttpConfigInput {
    fn from(value: NativeHttpConfig) -> Self {
        value.into_input()
    }
}
