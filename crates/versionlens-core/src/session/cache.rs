use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::atomic::Ordering;
use std::sync::{Arc, MutexGuard};
use std::time::Duration;

use versionlens_cache::{CacheKey, MemoryCache};
use versionlens_http::HttpConfig;
use versionlens_model::{Dependency, Ecosystem, ManifestKind};
use versionlens_suggestions::{Suggestion, UpdateChoice};

use super::operation::OperationContext;
use crate::ProviderCacheConfig;
use crate::VersionLensSession;
use crate::cache::{request_cache_key, suggestion_cache_key};
use crate::registry::RegistryContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct CachedLatest {
    pub(crate) latest: String,
    pub(crate) builds: Vec<String>,
    pub(crate) choices: Vec<UpdateChoice>,
    #[serde(default)]
    pub(crate) fixed_requirement_matched: Option<bool>,
}

impl VersionLensSession {
    pub(super) fn purge_expired_cache(&self) {
        self.synchronize_persistent_cache();
        self.result_state
            .parsed_dependencies
            .lock()
            .unwrap_or_else(crate::recover_poison)
            .purge_expired();
        self.cache().purge_expired();
        self.request_body_cache().purge_expired();
        self.suggestion_cache().purge_expired();
        self.vulnerability_cache().purge_expired();
    }

    pub fn clear_cache(&self) {
        let _ = self.try_clear_cache();
    }

    pub fn try_clear_cache(&self) -> std::io::Result<()> {
        let mut observed = self
            .storage_state
            .observed_persistent_epoch
            .lock()
            .unwrap_or_else(crate::recover_poison);
        self.storage_state
            .cache_epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        let result = self
            .storage_state
            .persistent_cache
            .as_ref()
            .map(|cache| cache.clear())
            .transpose();
        self.clear_memory_cache();
        if let Ok(epoch) = &result {
            *observed = *epoch;
        }
        drop(observed);
        result.map(|_| ())
    }

    pub(super) fn clear_memory_cache(&self) {
        self.clear_workspace_graphs();
        self.result_state
            .parsed_dependencies
            .lock()
            .unwrap_or_else(crate::recover_poison)
            .clear();
        self.cache().clear();
        self.request_body_cache().clear();
        self.request_state
            .request_locks
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .clear();
        self.suggestion_cache().clear();
        self.vulnerability_cache().clear();
        self.request_state
            .dotnet_registry_sources
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .take();
    }

    pub(crate) fn cache(&self) -> MutexGuard<'_, MemoryCache<CachedLatest>> {
        self.result_state
            .latest
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn request_body_cache(&self) -> MutexGuard<'_, MemoryCache<Arc<str>>> {
        self.request_state
            .request_body_cache
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn cached_request_body(&self, key: &CacheKey) -> Option<Arc<str>> {
        self.request_body_cache().get(key).cloned()
    }

    pub(crate) fn request_lock(&self, key: &CacheKey) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self
            .request_state
            .request_locks
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
        locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = locks.get(key).and_then(std::sync::Weak::upgrade) {
            return lock;
        }

        let lock = Arc::new(tokio::sync::Mutex::new(()));
        locks.insert(key.clone(), Arc::downgrade(&lock));
        lock
    }

    pub(crate) fn cache_request_body(
        &self,
        key: CacheKey,
        body: impl Into<Arc<str>>,
        ttl: Duration,
        operation: &OperationContext,
    ) {
        let mut cache = self.request_body_cache();
        if !operation.is_current() {
            return;
        }
        cache.insert_with_ttl(key, body.into(), ttl);
    }

    pub(crate) fn request_cache_key(&self, url: &str, config: &HttpConfig) -> CacheKey {
        request_cache_key(url, self.request_context_identity(config))
    }

    pub(crate) fn effective_http_config(
        &self,
        url: &str,
        ecosystem: Ecosystem,
        context: &RegistryContext,
    ) -> HttpConfig {
        let auth_headers = context.auth_headers_for_url(ecosystem, url);
        let base = self.http_config_with_headers(ecosystem, context.manifest_kind(), &auth_headers);
        context.http_config_for_request(ecosystem, url, base)
    }

    fn request_context_identity(&self, config: &HttpConfig) -> u128 {
        let mut first = self.request_state.request_context_hashers[0].build_hasher();
        0_u8.hash(&mut first);
        hash_http_config(config, &mut first);
        let mut second = self.request_state.request_context_hashers[1].build_hasher();
        1_u8.hash(&mut second);
        hash_http_config(config, &mut second);
        (u128::from(first.finish()) << 64) | u128::from(second.finish())
    }

    pub(crate) fn suggestion_cache(&self) -> MutexGuard<'_, MemoryCache<Suggestion>> {
        self.result_state
            .suggestions
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn cached_resolved_suggestion(
        &self,
        dependency: &Dependency,
        scope: &str,
    ) -> Option<Suggestion> {
        let key = suggestion_cache_key(dependency, scope);
        if let Some(value) = self.suggestion_cache().get(&key) {
            let mut suggestion = value.to_owned();
            suggestion.dependency = dependency.clone();
            return Some(suggestion);
        }
        let epoch = self.storage_state.cache_epoch.load(Ordering::Acquire);
        let persistent_key = self.persistent_suggestion_key(dependency, scope);
        let (suggestion, ttl) = self.load_persistent_suggestion(&persistent_key, dependency)?;
        self.synchronize_persistent_cache();
        let mut cache = self.suggestion_cache();
        if self.storage_state.cache_epoch.load(Ordering::Acquire) != epoch {
            return None;
        }
        cache.insert_with_ttl(key, suggestion.clone(), ttl);
        drop(cache);
        Some(suggestion)
    }

    pub(crate) fn cache_resolved_suggestions(
        &self,
        suggestions: &[Suggestion],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
        scope: &str,
    ) {
        let entries = suggestions
            .iter()
            .map(|suggestion| {
                (
                    suggestion_cache_key(&suggestion.dependency, scope),
                    self.persistent_suggestion_key(&suggestion.dependency, scope),
                    suggestion.to_owned(),
                    if suggestion.status == versionlens_suggestions::SuggestionStatus::Error {
                        self.cache_ttl(suggestion.dependency.ecosystem, manifest_kind)
                            .min(Duration::from_secs(30))
                    } else {
                        self.cache_ttl(suggestion.dependency.ecosystem, manifest_kind)
                    },
                )
            })
            .collect::<Vec<_>>();
        let mut cache = self.suggestion_cache();
        if !operation.is_current() {
            return;
        }
        for (key, _, suggestion, ttl) in &entries {
            cache.insert_with_ttl(key.clone(), suggestion.clone(), *ttl);
        }
        drop(cache);
        self.store_persistent_suggestions(
            entries
                .iter()
                .map(|(_, key, suggestion, ttl)| (key.clone(), suggestion, *ttl)),
            operation,
        );
    }

    pub(crate) fn cache_ttl(
        &self,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
    ) -> Duration {
        provider_cache_ttl(
            self.config.cache_ttl_ms,
            &self.config.providers.provider_cache,
            ecosystem,
            manifest_kind,
        )
    }
}

fn hash_http_config(config: &HttpConfig, hasher: &mut impl Hasher) {
    config.timeout_ms.hash(hasher);
    config.strict_ssl.hash(hasher);
    config.proxy.hash(hasher);
    config.ca_file.hash(hasher);
    config.ca.hash(hasher);
    config.cert_file.hash(hasher);
    config.key_file.hash(hasher);
    config.cert.hash(hasher);
    config.key.hash(hasher);
    for header in &config.auth_headers {
        header.name.hash(hasher);
        header.value.hash(hasher);
        header.url.hash(hasher);
    }
}

fn provider_cache_ttl(
    default_ttl_ms: u64,
    provider_cache: &[ProviderCacheConfig],
    ecosystem: Ecosystem,
    manifest_kind: Option<ManifestKind>,
) -> Duration {
    provider_cache
        .iter()
        .rfind(|config| config.ecosystem == ecosystem && config.applies_to_manifest(manifest_kind))
        .map(|config| crate::duration_from_millis(config.cache_ttl_ms))
        .unwrap_or_else(|| crate::duration_from_millis(default_ttl_ms))
}

pub(super) mod weights;

#[cfg(test)]
mod tests;
