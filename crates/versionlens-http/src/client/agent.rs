use std::collections::HashMap;
use std::fs::read;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use reqwest::tls::{Certificate, Identity};
use reqwest::{Client, Proxy};

use crate::config::HttpConfig;
use crate::error::HttpError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClientCacheKey {
    timeout_ms: u64,
    strict_ssl: bool,
    proxy: Option<String>,
}

static CLIENT_CACHE: OnceLock<Mutex<HashMap<ClientCacheKey, Client>>> = OnceLock::new();
static TLS_PROVIDER: OnceLock<()> = OnceLock::new();

pub(super) fn client(config: &HttpConfig) -> Result<Client, HttpError> {
    install_tls_provider();
    if let Some(key) = cache_key(config) {
        let cached = client_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&key)
            .cloned();
        if let Some(client) = cached {
            return Ok(client);
        }

        let client = build_client(config)?;
        let mut cache = client_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        return Ok(cache.entry(key).or_insert(client).clone());
    }

    build_client(config)
}

pub(super) fn install_tls_provider() {
    TLS_PROVIDER.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn client_cache() -> &'static Mutex<HashMap<ClientCacheKey, Client>> {
    CLIENT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_key(config: &HttpConfig) -> Option<ClientCacheKey> {
    if config.ca_file.is_some()
        || config.ca.is_some()
        || config.cert_file.is_some()
        || config.key_file.is_some()
        || config.cert.is_some()
        || config.key.is_some()
    {
        return None;
    }

    Some(ClientCacheKey {
        timeout_ms: config.timeout_ms,
        strict_ssl: config.strict_ssl,
        proxy: config.proxy.clone(),
    })
}

fn build_client(config: &HttpConfig) -> Result<Client, HttpError> {
    let mut builder = Client::builder()
        .tls_backend_rustls()
        .timeout(Duration::from_millis(config.timeout_ms))
        .tls_danger_accept_invalid_certs(!config.strict_ssl)
        .tls_danger_accept_invalid_hostnames(!config.strict_ssl);

    builder = match &config.proxy {
        Some(proxy) => builder.proxy(Proxy::all(proxy)?),
        None => builder.no_proxy(),
    };

    let ca = match (config.ca_file.as_deref(), config.ca.as_deref()) {
        (Some(path), _) => Some(read(path)?),
        (None, Some(ca)) => Some(ca.as_bytes().to_vec()),
        (None, None) => None,
    };
    if let Some(ca) = ca {
        builder = builder.tls_certs_only(Certificate::from_pem_bundle(&ca)?);
    }

    let cert = material(config.cert_file.as_deref(), config.cert.as_deref())?;
    let key = material(config.key_file.as_deref(), config.key.as_deref())?;
    if let (Some(mut cert), Some(key)) = (cert, key) {
        cert.extend_from_slice(&key);
        builder = builder.identity(Identity::from_pem(&cert)?);
    }

    Ok(builder.build()?)
}

fn material(path: Option<&str>, inline: Option<&str>) -> Result<Option<Vec<u8>>, HttpError> {
    match (path, inline) {
        (Some(path), _) => Ok(Some(read(path)?)),
        (None, Some(value)) => Ok(Some(value.as_bytes().to_vec())),
        (None, None) => Ok(None),
    }
}

#[cfg(test)]
pub(super) mod tests;
