use std::sync::MutexGuard;
use std::time::Duration;

use versionlens_cache::{CacheKey, MemoryCache};
use versionlens_parsers::{Dependency, Ecosystem, ManifestKind};
use versionlens_suggestions::{Suggestion, UpdateChoice};

use crate::ProviderCacheConfig;
use crate::VersionLensSession;
use crate::cache::{request_cache_key, suggestion_cache_key};

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
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.suggestion_cache().clear();
        self.vulnerability_cache().clear();
        self.dotnet_registry_sources
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }

    pub(crate) fn cache(&self) -> MutexGuard<'_, MemoryCache<CachedLatest>> {
        self.latest_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn cached_latest(&self, key: &CacheKey) -> Option<String> {
        self.cache()
            .get(key)
            .map(|cached| String::from(cached.latest.as_str()))
    }

    pub(crate) fn request_body_cache(&self) -> MutexGuard<'_, MemoryCache<String>> {
        self.request_body_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn cached_request_body(&self, url: &str) -> Option<String> {
        self.request_body_cache()
            .get(&request_cache_key(url))
            .map(|body| String::from(body.as_str()))
    }

    pub(crate) fn request_lock(&self, url: &str) -> std::sync::Arc<std::sync::Mutex<()>> {
        let mut locks = self
            .request_locks
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        locks
            .entry(url.to_owned())
            .or_insert_with(|| std::sync::Arc::new(std::sync::Mutex::new(())))
            .clone()
    }

    pub(crate) fn cache_request_body(
        &self,
        url: &str,
        body: &str,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
    ) {
        self.request_body_cache().insert_with_ttl(
            request_cache_key(url),
            body.to_owned(),
            self.cache_ttl(ecosystem, manifest_kind),
        );
    }

    pub(crate) fn suggestion_cache(&self) -> MutexGuard<'_, MemoryCache<Suggestion>> {
        self.suggestion_cache
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn cached_resolved_suggestion(&self, dependency: &Dependency) -> Option<Suggestion> {
        self.suggestion_cache()
            .get(&suggestion_cache_key(dependency))
            .map(ToOwned::to_owned)
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

fn provider_cache_ttl(
    default_ttl_ms: u64,
    provider_cache: &[ProviderCacheConfig],
    ecosystem: Ecosystem,
    manifest_kind: Option<ManifestKind>,
) -> Duration {
    provider_cache
        .iter()
        .rfind(|config| config.ecosystem == ecosystem && config.applies_to_manifest(manifest_kind))
        .map(|config| Duration::from_millis(config.cache_ttl_ms))
        .unwrap_or_else(|| Duration::from_millis(default_ttl_ms))
}

#[cfg(test)]
mod tests;
