use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use versionlens_cache::MemoryCache;
use versionlens_parsers::{Ecosystem, ManifestKind};
use versionlens_providers::VulnerabilityAdvisory;
use versionlens_suggestions::Suggestion;

use crate::config::SessionConfig;
use crate::model::AuthorizationRequestPayload;
use cache::CachedLatest;

mod cache;
mod classify;
mod commands;
mod dependencies;
mod documents;
mod presentation;
mod resolution;

#[derive(Debug)]
pub struct VersionLensSession {
    pub(crate) config: SessionConfig,
    pub(crate) latest_cache: Mutex<MemoryCache<CachedLatest>>,
    pub(crate) request_body_cache: Mutex<MemoryCache<String>>,
    pub(crate) request_locks: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    pub(crate) suggestion_cache: Mutex<MemoryCache<Suggestion>>,
    pub(crate) vulnerability_cache: Mutex<MemoryCache<Vec<VulnerabilityAdvisory>>>,
    pub(crate) dotnet_registry_sources: Mutex<Option<Vec<String>>>,
    authorization_requests: Mutex<Vec<AuthorizationRequestPayload>>,
}

impl VersionLensSession {
    pub fn new(config: SessionConfig) -> Self {
        let cache_ttl = Duration::from_millis(config.cache_ttl_ms);
        Self {
            config,
            latest_cache: Mutex::new(MemoryCache::new(cache_ttl)),
            request_body_cache: Mutex::new(MemoryCache::new(cache_ttl)),
            request_locks: Mutex::new(HashMap::new()),
            suggestion_cache: Mutex::new(MemoryCache::new(cache_ttl)),
            vulnerability_cache: Mutex::new(MemoryCache::new(cache_ttl)),
            dotnet_registry_sources: Mutex::new(None),
            authorization_requests: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn clear_authorization_requests(&self) {
        let mut requests = match self.authorization_requests.lock() {
            Ok(requests) => requests,
            Err(poisoned) => poisoned.into_inner(),
        };
        requests.clear();
    }

    pub(crate) fn record_authorization_request(&self, auth_url: String, request_url: String) {
        let mut requests = match self.authorization_requests.lock() {
            Ok(requests) => requests,
            Err(poisoned) => poisoned.into_inner(),
        };
        if requests
            .iter()
            .any(|request| request.auth_url == auth_url && request.request_url == request_url)
        {
            return;
        }
        requests.push(AuthorizationRequestPayload {
            auth_url,
            request_url,
        });
    }

    pub(crate) fn take_authorization_requests(&self) -> Vec<AuthorizationRequestPayload> {
        let mut requests = match self.authorization_requests.lock() {
            Ok(requests) => requests,
            Err(poisoned) => poisoned.into_inner(),
        };
        requests.drain(..).collect()
    }

    pub(crate) fn provider_enabled_for_manifest(
        &self,
        kind: ManifestKind,
        ecosystem: Ecosystem,
    ) -> bool {
        self.config.enabled_providers.is_empty()
            || self
                .config
                .enabled_providers
                .iter()
                .any(|provider| provider.applies_to_manifest(kind, ecosystem))
    }
}
