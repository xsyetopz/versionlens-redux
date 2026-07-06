use std::collections::HashMap;
use std::fs::read;
use std::sync::{Mutex, OnceLock};
use ureq::Error as UreqError;

use ureq::Agent;
use ureq::tls::PemItem::Certificate as PemCertificate;
use ureq::tls::{Certificate, ClientCert, PrivateKey, RootCerts, TlsConfig, parse_pem};

use crate::config::HttpConfig;
use crate::error::HttpError;

type StaticCertificates = Vec<Certificate<'static>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AgentCacheKey {
    timeout_ms: u64,
    strict_ssl: bool,
    proxy: Option<String>,
}

static AGENT_CACHE: OnceLock<Mutex<HashMap<AgentCacheKey, Agent>>> =
    <OnceLock<Mutex<HashMap<AgentCacheKey, Agent>>>>::new();

pub(super) fn agent(config: &HttpConfig) -> Result<Agent, HttpError> {
    if let Some(key) = cache_key(config) {
        let cached_agent = agent_cache()
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .get(&key)
            .cloned();
        if let Some(agent) = cached_agent {
            return Ok(agent);
        }

        let agent = build_agent(config)?;
        let mut cache = agent_cache()
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
        return Ok(cache.entry(key).or_insert(agent).clone());
    }

    build_agent(config)
}

fn agent_cache() -> &'static Mutex<HashMap<AgentCacheKey, Agent>> {
    AGENT_CACHE.get_or_init(|| crate::mutex(<HashMap<AgentCacheKey, Agent>>::new()))
}

fn cache_key(config: &HttpConfig) -> Option<AgentCacheKey> {
    if config.ca_file.is_some()
        || config.ca.is_some()
        || config.cert_file.is_some()
        || config.key_file.is_some()
        || config.cert.is_some()
        || config.key.is_some()
    {
        return None;
    }

    Some(AgentCacheKey {
        timeout_ms: config.timeout_ms,
        strict_ssl: config.strict_ssl,
        proxy: config.proxy.as_ref().map(|value| value.to_owned()),
    })
}

fn build_agent(config: &HttpConfig) -> Result<Agent, HttpError> {
    let timeout_ms = config.timeout_ms;
    let mut builder = ureq::config::Config::builder()
        .timeout_global(Some(crate::duration_from_millis(timeout_ms)));

    builder = match &config.proxy {
        Some(proxy) => builder.proxy(Some(ureq::Proxy::new(proxy)?)),
        None => builder.proxy(None),
    };

    builder = builder.tls_config(tls_config(config)?);

    Ok(<Agent>::new_with_config(builder.build()))
}

#[cfg(test)]
pub(super) fn uses_same_agent_cache_key(first: &HttpConfig, second: &HttpConfig) -> bool {
    cache_key(first) == cache_key(second)
}

#[cfg(test)]
pub(super) fn uses_agent_cache(config: &HttpConfig) -> bool {
    cache_key(config).is_some()
}

fn root_certs_from_certs(certs: &StaticCertificates) -> RootCerts {
    <RootCerts>::new_with_certs(certs)
}

fn private_key_from_pem(bytes: &[u8]) -> Result<PrivateKey<'static>, UreqError> {
    <PrivateKey<'_>>::from_pem(bytes)
}

fn tls_config(config: &HttpConfig) -> Result<TlsConfig, HttpError> {
    let mut builder = <TlsConfig>::builder();
    if !config.strict_ssl {
        builder = builder.disable_verification(true);
    }
    if let Some(path) = config.ca_file.as_deref() {
        builder = builder.root_certs(root_certs_from_certs(&ca_file_certs(path)?));
    } else if let Some(ca) = config.ca.as_deref() {
        builder = builder.root_certs(root_certs_from_certs(&pem_certs(ca.as_bytes())?));
    }

    let certs = match (config.cert_file.as_deref(), config.cert.as_deref()) {
        (Some(cert_file), _) => Some(ca_file_certs(cert_file)?),
        (None, Some(cert)) => Some(pem_certs(cert.as_bytes())?),
        (None, None) => None,
    };
    let key = match (config.key_file.as_deref(), config.key.as_deref()) {
        (Some(key_file), _) => Some(private_key_file(key_file)?),
        (None, Some(key)) => Some(private_key_from_pem(key.as_bytes())?),
        (None, None) => None,
    };
    if let (Some(certs), Some(key)) = (certs, key) {
        builder = builder.client_cert(Some(<ClientCert>::new_with_certs(&certs, key)));
    }

    Ok(builder.build())
}

fn ca_file_certs(path: &str) -> Result<StaticCertificates, HttpError> {
    let bytes = read(path)?;
    pem_certs(&bytes)
}

fn pem_certs(bytes: &[u8]) -> Result<StaticCertificates, HttpError> {
    let mut certs = vec![];
    for item in parse_pem(bytes) {
        if let PemCertificate(certificate) = item? {
            certs.push(certificate);
        }
    }
    if certs.is_empty() {
        Err(ureq::Error::Tls("No pem encoded cert found").into())
    } else {
        Ok(certs)
    }
}

fn private_key_file(path: &str) -> Result<PrivateKey<'static>, HttpError> {
    let bytes = read(path)?;
    Ok(private_key_from_pem(&bytes)?)
}
