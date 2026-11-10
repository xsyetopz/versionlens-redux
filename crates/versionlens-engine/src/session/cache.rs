use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::PoisonError;
use std::sync::atomic::Ordering;
use std::sync::{Arc, MutexGuard};
use std::time::Duration;

use versionlens_fetch::HttpConfig;
use versionlens_fetch::{CacheKey, MemoryCache};
use versionlens_model::{Dependency, Ecosystem, ManifestKind};
use versionlens_update::{Suggestion, UpdateChoice};

use super::operation::OperationContext;
use super::storage::CacheScope;
use crate::ProviderCacheConfig;
use crate::VersionLensSession;
use crate::cache::request_cache_key;
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
            .unwrap_or_else(PoisonError::into_inner)
            .purge_expired();
        self.cache().purge_expired();
        self.request_body_cache().purge_expired();
        self.suggestion_cache().purge_expired();
        self.vulnerability_cache().purge_expired();
    }

    /// Clears the memory cache and the persistent cache. A failure to clear the
    /// persistent cache is ignored.
    pub fn clear_cache(&self) {
        drop(self.try_clear_cache());
    }

    /// Clears the memory cache and the persistent cache.
    ///
    /// # Errors
    ///
    /// Returns the error of the persistent cache when it cannot be cleared.
    pub fn try_clear_cache(&self) -> std::io::Result<()> {
        let mut observed = self
            .storage_state
            .observed_persistent_epoch
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        self.storage_state
            .cache_epoch
            .fetch_add(1, Ordering::AcqRel);
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
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        self.cache().clear();
        self.request_body_cache().clear();
        self.request_state
            .request_locks
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clear();
        self.suggestion_cache().clear();
        self.vulnerability_cache().clear();
        self.request_state
            .dotnet_registry_sources
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    }

    pub(crate) fn cache(&self) -> MutexGuard<'_, MemoryCache<CachedLatest>> {
        self.result_state
            .latest
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn request_body_cache(&self) -> MutexGuard<'_, MemoryCache<Arc<str>>> {
        self.request_state
            .request_body_cache
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn cached_request_body(&self, key: &CacheKey) -> Option<Arc<str>> {
        self.request_body_cache().get(key).cloned()
    }

    pub(crate) fn request_lock(&self, key: &CacheKey) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self
            .request_state
            .request_locks
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
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
        let mut config = context.http_config_for_request(ecosystem, url, base);
        if let Some(credential) = self.request_state.credentials.resolve(url) {
            versionlens_fetch::apply_credential(&mut config.auth_headers, &credential);
        }
        config
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
            .unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn cached_resolved_suggestion(
        &self,
        dependency: &Dependency,
        scope: &CacheScope,
    ) -> Option<Suggestion> {
        let key = scope.suggestion_key(dependency);
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
        scope: &CacheScope,
    ) {
        let entries = suggestions
            .iter()
            .map(|suggestion| {
                (
                    scope.suggestion_key(&suggestion.dependency),
                    self.persistent_suggestion_key(&suggestion.dependency, scope),
                    suggestion.to_owned(),
                    if matches!(
                        suggestion.status,
                        versionlens_update::SuggestionStatus::Error
                            | versionlens_update::SuggestionStatus::RateLimited
                    ) {
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
        .map_or_else(
            || Duration::from_millis(default_ttl_ms),
            |config| Duration::from_millis(config.cache_ttl_ms),
        )
}

pub(super) mod weights;

#[cfg(test)]
mod tests {
    use crate::RegistryResponseInput;
    use crate::{ProviderCacheConfig, ProviderSettings, SessionConfig};
    use std::thread::sleep;
    use std::time::Duration;

    use versionlens_model::DocumentInput;

    use versionlens_fetch::HttpHeader;
    use versionlens_model::Ecosystem::Npm;
    use versionlens_model::ManifestKind::{NpmPackageJson, PnpmYaml};

    fn session_with_manifest_cache(
        manifest_kind: Option<versionlens_model::ManifestKind>,
    ) -> crate::VersionLensSession {
        crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings {
                provider_cache: vec![ProviderCacheConfig {
                    ecosystem: Npm,
                    manifest_kind,
                    cache_ttl_ms: 200,
                }],
                ..Default::default()
            },
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: false,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        })
    }

    #[test]
    fn request_cache_identity_separates_effective_security_context_without_exposing_secrets() {
        let session = crate::test_support::test_session(false);
        let mut first = versionlens_fetch::standard_http_config();
        first.auth_headers.push(HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer first-secret".to_owned(),
            url: None,
        });
        let mut second = first.clone();
        second.auth_headers[0].value = "Bearer second-secret".to_owned();
        let mut proxied = first.clone();
        proxied.proxy = Some("http://proxy.example.test".to_owned());
        let mut custom_tls = first.clone();
        custom_tls.strict_ssl = false;
        custom_tls.ca = Some("private-ca-material".to_owned());

        let first_key = session.request_cache_key("https://registry.example.test/pkg", &first);
        let second_key = session.request_cache_key("https://registry.example.test/pkg", &second);
        let proxied_key = session.request_cache_key("https://registry.example.test/pkg", &proxied);
        let custom_tls_key =
            session.request_cache_key("https://registry.example.test/pkg", &custom_tls);

        assert_ne!(first_key, second_key);
        assert_ne!(first_key, proxied_key);
        assert_ne!(first_key, custom_tls_key);
        assert!(!first_key.as_str().contains("first-secret"));
        assert!(!second_key.as_str().contains("second-secret"));
        assert!(!custom_tls_key.as_str().contains("private-ca-material"));

        session.cache_request_body(
            first_key.clone(),
            "first response",
            session.cache_ttl(Npm, None),
            &crate::session::operation::OperationContext::default(),
        );
        assert_eq!(
            session.cached_request_body(&first_key).as_deref(),
            Some("first response")
        );
        assert!(session.cached_request_body(&second_key).is_none());
        assert!(session.cached_request_body(&proxied_key).is_none());
        assert!(session.cached_request_body(&custom_tls_key).is_none());
    }

    #[test]
    fn completed_request_lock_keys_are_pruned_during_subsequent_requests() {
        let session = crate::test_support::test_session(false);
        let http = versionlens_fetch::standard_http_config();

        for index in 0..100 {
            let key = session.request_cache_key(
                &format!("https://registry.example.test/package-{index}"),
                &http,
            );
            drop(session.request_lock(&key));
        }

        assert_eq!(session.request_state.request_locks.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn provider_cache_overrides_global_cache_ttl() {
        let session = session_with_manifest_cache(None);

        let input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("provider-cache-overrides-global-cache-ttl.json"),
            None,
        );
        let responses = [RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        )];

        session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        sleep(Duration::from_millis(400));

        assert_eq!(session.analyze_document(input).code_lenses.len(), 0);
    }

    #[test]
    fn manifest_scoped_provider_cache_does_not_override_package_json_npm() {
        let session = session_with_manifest_cache(Some(PnpmYaml));

        assert_eq!(
            session.cache_ttl(Npm, Some(NpmPackageJson)),
            Duration::from_millis(300_000)
        );
        assert_eq!(
            session.cache_ttl(Npm, Some(PnpmYaml)),
            Duration::from_millis(200)
        );
    }

    #[tokio::test]
    async fn manifest_scoped_provider_cache_controls_cached_suggestions() {
        let session = session_with_manifest_cache(Some(PnpmYaml));
        let input = DocumentInput::new(
            "file:///pnpm-workspace.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("manifest-scoped-provider-cache-controls-cached-suggestions.yaml"),
            None,
        );

        crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[crate::test_support::npm_latest_response(
                "left-pad", "1.1.0",
            )],
        )
        .await;
        assert_eq!(
            session.analyze_document(input.clone()).code_lenses[1].title,
            "↑  latest 1.1.0"
        );

        sleep(Duration::from_millis(400));

        assert_eq!(session.analyze_document(input).code_lenses.len(), 0);
    }

    #[tokio::test]
    async fn registry_responses_override_cached_latest_version() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("registry-responses-override-cached-latest-version.json"),
            None,
        );

        session
            .resolve_document_with_responses(
                input.clone(),
                &[crate::test_support::npm_latest_response(
                    "left-pad", "1.1.0",
                )],
            )
            .await;
        let refreshed = session
            .resolve_document_with_responses(
                input,
                &[crate::test_support::npm_latest_response(
                    "left-pad", "1.2.0",
                )],
            )
            .await;

        assert_eq!(refreshed.edits[0].new_text, "1.2.0");
    }

    #[tokio::test]
    async fn caches_latest_version_and_clear_cache_removes_it() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("caches-latest-version-and-clear-cache-removes-it.json"),
            None,
        );

        let first = session
            .resolve_document_with_responses(
                input.clone(),
                &[crate::test_support::npm_latest_response(
                    "left-pad", "1.1.0",
                )],
            )
            .await;
        let cached = session
            .resolve_document_with_responses(input.clone(), &[])
            .await;

        session.clear_cache();
        let cleared = session.analyze_document(input);

        assert_eq!(first.edits[0].new_text, "1.1.0");
        assert_eq!(cached.edits[0].new_text, "1.1.0");
        assert_eq!(cleared.diagnostics.len(), 0);
    }

    #[tokio::test]
    async fn cached_latest_preserves_registry_build_aliases() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("latest-preserves-registry-build-aliases.json"),
            None,
        );
        let response = RegistryResponseInput::new("left-pad".to_owned(), Npm, r#"{"dist-tags":{"latest":"1.0.0+build.2"},"versions":{"1.0.0":{},"1.0.0+build.1":{},"1.0.0+build.2":{}}}"#
            .to_owned());

        let first = session
            .resolve_document_with_responses(input.clone(), &[response])
            .await;
        let cached = session.resolve_document_with_responses(input, &[]).await;

        assert_eq!(first.suggestions[0].status, "current");
        assert_eq!(cached.suggestions[0].status, "current");
        assert_eq!(cached.suggestions[0].builds, first.suggestions[0].builds);
    }

    #[test]
    fn clear_cache_removes_dotnet_registry_sources() {
        let session = crate::test_support::test_session(true);

        *session
            .request_state
            .dotnet_registry_sources
            .lock()
            .unwrap() = Some(vec!["https://nuget.test/v3/index.json".to_owned()]);

        session.clear_cache();

        assert!(
            session
                .request_state
                .dotnet_registry_sources
                .lock()
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn analyze_document_uses_cached_latest_for_code_lens_title() {
        let session = crate::test_support::test_session(true);
        let input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("analyze-document-uses-cached-latest-for-code-lens-title.json"),
            None,
        );

        session
            .resolve_document_with_responses(
                input.clone(),
                &[crate::test_support::npm_latest_response(
                    "left-pad", "1.1.0",
                )],
            )
            .await;
        let output = session.analyze_document(input);

        assert_eq!(output.code_lenses[0].title, "🟡 fixed 1.0.0");
        assert_eq!(output.code_lenses[0].action_kind(), "");
        assert_eq!(output.code_lenses[1].title, "↑  latest 1.1.0");
        assert_eq!(output.code_lenses[1].action_kind(), "UpdateDependency");
        assert_eq!(output.code_lenses[1].arguments()[0], "left-pad");
        assert!(output.code_lenses[1].arguments()[1].starts_with("left-pad"));
    }

    #[tokio::test]
    async fn cached_latest_is_scoped_to_dependency_requirement_for_update_choices() {
        let session = crate::test_support::test_session(false);
        let fixed_input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(
                "latest-is-scoped-to-dependency-requirement-for-update-choices.json",
            ),
            None,
        );
        let range_input = DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("latest-is-scoped-to-dependency-requirement-for-range.json"),
            None,
        );

        session
            .resolve_document_with_responses(
                fixed_input,
                &[RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{
              "dist-tags": { "latest": "2.0.0" },
              "versions": {
                "1.0.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "2.0.0": {}
              }
            }"#
                    .to_owned(),
                )],
            )
            .await;

        let cached_range = session
            .resolve_document_with_responses(range_input.clone(), &[])
            .await;
        let analysis = session.analyze_document(range_input);
        let titles = analysis
            .code_lenses
            .iter()
            .map(|lens| lens.title.as_str())
            .collect::<Vec<_>>();
        let arguments = crate::test_support::all_code_lens_arguments(&analysis);

        assert_eq!(cached_range.suggestions[0].status, "satisfies");
        assert_eq!(
            titles,
            ["🟡 satisfies 1.1.2", "↑  bump 1.1.2", "↑  latest 2.0.0"]
        );
        assert_eq!(
            arguments,
            [
                Vec::<&str>::new(),
                vec!["update", "1.1.2"],
                vec!["update", "2.0.0"]
            ]
        );
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/session/cache", name)
    }

    #[test]
    fn clearing_shared_session_invalidates_inflight_cache_writes() {
        let session = crate::test_support::test_session(false);
        let operation = crate::session::operation::OperationContext::default()
            .with_generation(&session.storage_state.cache_epoch, None);
        let key = session.request_cache_key("https://example.test/package", &session.config.http);
        let other = session.clone();
        other.clear_cache();
        session.cache_request_body(
            key.clone(),
            "response",
            session.cache_ttl(Npm, None),
            &operation,
        );
        assert!(!operation.is_current());
        assert!(other.cached_request_body(&key).is_none());
    }
}
