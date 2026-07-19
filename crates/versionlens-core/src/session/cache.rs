use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use versionlens_cache::{CacheKey, MemoryCache};
use versionlens_http::HttpConfig;
use versionlens_model::{Dependency, Ecosystem, ManifestKind};
use versionlens_suggestions::{Suggestion, UpdateChoice};

use crate::ProviderCacheConfig;
use crate::VersionLensSession;
use crate::cache::{request_cache_key, suggestion_cache_key};
use crate::registry::RegistryContext;

#[derive(Debug)]
pub(crate) struct CachedLatest {
    pub(crate) latest: String,
    pub(crate) builds: Vec<String>,
    pub(crate) choices: Vec<UpdateChoice>,
}

impl VersionLensSession {
    pub fn clear_cache(&self) {
        self.cache().clear();
        self.request_body_cache().clear();
        self.request_locks
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .clear();
        self.suggestion_cache().clear();
        self.vulnerability_cache().clear();
        self.dotnet_registry_sources
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
            .take();
    }

    pub(crate) fn cache(&self) -> MutexGuard<'_, MemoryCache<CachedLatest>> {
        self.latest_cache
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn cached_latest(&self, key: &CacheKey) -> Option<String> {
        self.cache()
            .get(key)
            .map(|cached| cached.latest.as_str().to_owned())
    }

    pub(crate) fn request_body_cache(&self) -> MutexGuard<'_, MemoryCache<String>> {
        self.request_body_cache
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn cached_request_body(&self, key: &CacheKey) -> Option<String> {
        self.request_body_cache()
            .get(key)
            .map(|body| body.as_str().to_owned())
    }

    pub(crate) fn request_lock(&self, key: &CacheKey) -> Arc<Mutex<()>> {
        let mut locks = self
            .request_locks
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned));
        locks.retain(|_, lock| lock.strong_count() > 0);
        if let Some(lock) = locks.get(key).and_then(std::sync::Weak::upgrade) {
            return lock;
        }

        let lock = crate::arc(crate::mutex(()));
        locks.insert(key.clone(), Arc::downgrade(&lock));
        lock
    }

    pub(crate) fn cache_request_body(
        &self,
        key: CacheKey,
        body: &str,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
    ) {
        self.request_body_cache().insert_with_ttl(
            key,
            body.to_owned(),
            self.cache_ttl(ecosystem, manifest_kind),
        );
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
        let mut first = self.request_context_hashers[0].build_hasher();
        0_u8.hash(&mut first);
        hash_http_config(config, &mut first);
        let mut second = self.request_context_hashers[1].build_hasher();
        1_u8.hash(&mut second);
        hash_http_config(config, &mut second);
        (u128::from(first.finish()) << 64) | u128::from(second.finish())
    }

    pub(crate) fn suggestion_cache(&self) -> MutexGuard<'_, MemoryCache<Suggestion>> {
        self.suggestion_cache
            .lock()
            .unwrap_or_else(|poisoned| crate::recover_poison(poisoned))
    }

    pub(crate) fn cached_resolved_suggestion(&self, dependency: &Dependency) -> Option<Suggestion> {
        self.suggestion_cache()
            .get(&suggestion_cache_key(dependency))
            .map(|value| value.to_owned())
    }

    pub(crate) fn cache_resolved_suggestions(
        &self,
        suggestions: &[Suggestion],
        manifest_kind: Option<ManifestKind>,
    ) {
        let entries = suggestions
            .iter()
            .map(|suggestion| {
                (
                    suggestion_cache_key(&suggestion.dependency),
                    suggestion.to_owned(),
                    self.cache_ttl(suggestion.dependency.ecosystem, manifest_kind),
                )
            })
            .collect::<Vec<_>>();
        let mut cache = self.suggestion_cache();
        for (key, suggestion, ttl) in entries {
            cache.insert_with_ttl(key, suggestion, ttl);
        }
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

#[cfg(test)]
mod tests;
