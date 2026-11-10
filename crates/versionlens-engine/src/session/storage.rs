use std::path::Path;
use std::sync::PoisonError;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use versionlens_fetch::{PersistentCache, PersistentRecord};
use versionlens_model::{CanonicalReference, Dependency};
use versionlens_update::{Suggestion, SuggestionStatus, UpdateChoice};

use super::cache::CachedLatest;
use super::operation::{OperationContext, timestamp_ms};
use crate::registry::RegistryContext;
use crate::vulnerability::{VulnerabilityCheck, supports_vulnerability_check};
use crate::{VersionLensSession, cache, project};

/// Cache scope of one document. Registry results use the shared part. Results
/// that read other workspace files, such as local actions and project
/// versions, also name the workspace revision, so an unsaved edit misses them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CacheScope {
    shared: String,
    local: String,
}

impl CacheScope {
    pub(crate) fn suggestion_key(&self, dependency: &Dependency) -> versionlens_fetch::CacheKey {
        let scope = if reads_workspace(dependency) {
            &self.local
        } else {
            &self.shared
        };
        cache::suggestion_cache_key(dependency, scope)
    }
}

/// True when a dependency resolves from workspace files and not a registry.
pub(crate) fn reads_workspace(dependency: &Dependency) -> bool {
    project::is_project_version_dependency(dependency)
        || matches!(
            dependency.canonical_reference,
            Some(CanonicalReference::GitHubActionLocal { .. })
        )
}

struct PersistentOutcome<'a, T> {
    value: &'a T,
    failed: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedSuggestion {
    latest: Option<String>,
    resolved: Option<String>,
    status: SuggestionStatus,
    builds: Vec<String>,
    choices: Vec<CachedSuggestionChoice>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedSuggestionChoice {
    label: String,
    version: String,
    command: String,
    replacement: Option<CachedReplacement>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum CachedReplacement {
    Literal { value: String },
    GitHubActionSha { commit: String, tag: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "camelCase")]
enum CachedVulnerabilityCheck {
    Checked(Vec<CachedVulnerabilityAdvisory>),
    Unsupported,
    Failed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CachedVulnerabilityAdvisory {
    id: String,
    title: String,
    url: Option<String>,
}

impl CachedVulnerabilityCheck {
    fn from_check(check: &VulnerabilityCheck) -> Self {
        match check {
            VulnerabilityCheck::Checked(advisories) => Self::Checked(
                advisories
                    .iter()
                    .map(|advisory| CachedVulnerabilityAdvisory {
                        id: advisory.id.clone(),
                        title: advisory.title.clone(),
                        url: advisory.url.clone(),
                    })
                    .collect(),
            ),
            VulnerabilityCheck::Unsupported => Self::Unsupported,
            VulnerabilityCheck::Failed(message) => Self::Failed(message.clone()),
        }
    }

    fn into_check(self) -> VulnerabilityCheck {
        match self {
            Self::Checked(advisories) => VulnerabilityCheck::Checked(
                advisories
                    .into_iter()
                    .map(|advisory| versionlens_providers::VulnerabilityAdvisory {
                        id: advisory.id,
                        title: advisory.title,
                        url: advisory.url,
                    })
                    .collect(),
            ),
            Self::Unsupported => VulnerabilityCheck::Unsupported,
            Self::Failed(message) => VulnerabilityCheck::Failed(message),
        }
    }
}

impl CachedSuggestion {
    fn from_suggestion(suggestion: &Suggestion) -> Self {
        Self {
            latest: suggestion.latest.clone(),
            resolved: suggestion.resolved.clone(),
            status: suggestion.status,
            builds: suggestion.builds.clone(),
            choices: suggestion
                .choices
                .iter()
                .map(|choice| CachedSuggestionChoice::from_choice(choice, &suggestion.dependency))
                .collect(),
        }
    }

    fn into_suggestion(self, dependency: &Dependency) -> Suggestion {
        Suggestion {
            dependency: dependency.clone(),
            latest: self.latest,
            resolved: self.resolved,
            status: self.status,
            builds: self.builds,
            choices: self
                .choices
                .into_iter()
                .map(|choice| choice.into_choice(dependency))
                .collect(),
        }
    }
}

impl CachedSuggestionChoice {
    fn from_choice(choice: &UpdateChoice, dependency: &Dependency) -> Self {
        let replacement = choice.replacement.as_deref().map(|value| {
            if let Some(CanonicalReference::GitHubActionSha { separator, .. }) =
                dependency.canonical_reference.as_ref()
                && let Some((commit, tag)) = value.split_once(separator)
            {
                return CachedReplacement::GitHubActionSha {
                    commit: commit.to_owned(),
                    tag: tag.to_owned(),
                };
            }
            CachedReplacement::Literal {
                value: value.to_owned(),
            }
        });
        Self {
            label: choice.label.clone(),
            version: choice.version.clone(),
            command: choice.command.clone(),
            replacement,
        }
    }

    fn into_choice(self, dependency: &Dependency) -> UpdateChoice {
        let replacement = self.replacement.and_then(|replacement| match replacement {
            CachedReplacement::Literal { value } => Some(value),
            CachedReplacement::GitHubActionSha { commit, tag } => {
                let CanonicalReference::GitHubActionSha { separator, .. } =
                    dependency.canonical_reference.as_ref()?
                else {
                    return None;
                };
                Some(format!("{commit}{separator}{tag}"))
            }
        });
        UpdateChoice {
            label: self.label,
            version: self.version,
            command: self.command,
            replacement,
        }
    }
}

impl VersionLensSession {
    pub fn with_persistent_cache(mut self, directory: impl AsRef<Path>) -> std::io::Result<Self> {
        let cache = PersistentCache::open(directory)?;
        self.storage_state.observed_persistent_epoch = Arc::new(Mutex::new(Some(cache.epoch()?)));
        self.storage_state.persistent_cache = Some(Arc::new(cache));
        Ok(self)
    }

    pub(crate) fn synchronize_persistent_cache(&self) -> Option<u64> {
        let cache = self.storage_state.persistent_cache.as_ref()?;
        let mut observed = self
            .storage_state
            .observed_persistent_epoch
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let epoch = cache.epoch().ok()?;
        if *observed != Some(epoch) {
            self.storage_state
                .cache_epoch
                .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
            self.clear_memory_cache();
            *observed = Some(epoch);
        }
        drop(observed);
        Some(epoch)
    }

    pub(crate) fn operation_context(&self) -> OperationContext {
        let epoch = self.synchronize_persistent_cache();
        OperationContext::with_timeout(Duration::from_millis(self.config.http.timeout_ms))
            .with_generation(
                &self.storage_state.cache_epoch,
                self.storage_state.task_epoch,
            )
            .with_cancellation(self.storage_state.task_cancellation.clone())
            .with_storage(self.storage_state.persistent_cache.clone(), epoch)
    }

    pub(crate) fn cache_scope(&self, context: &RegistryContext) -> String {
        use std::hash::BuildHasher;
        let source = format!("{context:?}");
        if let Some(cache) = &self.storage_state.persistent_cache
            && let Ok(policy) = serde_json::to_vec(&self.config)
        {
            return cache.partition_key(&[b"scope-v1", source.as_bytes(), &policy]);
        }
        format!(
            "{:016x}{:016x}",
            self.request_state.request_context_hashers[0].hash_one(&source),
            self.request_state.request_context_hashers[1].hash_one(&source)
        )
    }

    pub(crate) fn document_cache_scope(
        &self,
        context: &RegistryContext,
        input: &versionlens_model::DocumentInput,
    ) -> CacheScope {
        self.document_cache_scope_with_graph(context, input).0
    }

    /// Cache scope plus the workspace graph whose fingerprint it names.
    /// Resolution uses this graph, so a result never lands under the
    /// fingerprint of a different graph.
    pub(crate) fn document_cache_scope_with_graph(
        &self,
        context: &RegistryContext,
        input: &versionlens_model::DocumentInput,
    ) -> (CacheScope, crate::workspace::WorkspaceGraph) {
        // Read the revision before any workspace file, so a result computed
        // from older files never lands under a newer revision.
        let revision = self.workspace_revision();
        let workspace = self.workspace_graph(input);
        let scope = serde_json::json!([
            self.cache_scope(context),
            input.uri,
            input.workspace_root,
            workspace
                .fingerprint
                .as_ref()
                .map(versionlens_fetch::CacheKey::as_str),
        ])
        .to_string();
        let scope = CacheScope {
            local: format!("{scope}|workspace-{revision}"),
            shared: scope,
        };
        self.hydrate_persistent_outcomes(input, &scope);
        (scope, workspace)
    }

    pub(crate) fn persistent_suggestion_key(
        &self,
        dependency: &Dependency,
        scope: &CacheScope,
    ) -> String {
        if reads_workspace(dependency) {
            return String::new();
        }
        let Some(cache) = &self.storage_state.persistent_cache else {
            return String::new();
        };
        let canonical = persistent_canonical_identity(dependency.canonical_reference.as_ref());
        let identity = serde_json::json!([
            dependency.name,
            dependency.requirement,
            dependency.ecosystem,
            dependency.group,
            dependency.hosted_url,
            dependency.hosted_name,
            canonical,
        ]);
        cache.partition_key(&[
            b"suggestion-v1",
            scope.shared.as_bytes(),
            identity.to_string().as_bytes(),
        ])
    }

    pub(crate) fn load_persistent_suggestion(
        &self,
        key: &str,
        dependency: &Dependency,
    ) -> Option<(Suggestion, Duration)> {
        if key.is_empty() {
            return None;
        }
        let now = timestamp_ms();
        let record = self
            .storage_state
            .persistent_cache
            .as_ref()?
            .get(key, now)
            .ok()??;
        suggestion_record(record, dependency, now)
    }

    pub(crate) fn store_persistent_suggestions<'a>(
        &self,
        entries: impl Iterator<Item = (String, &'a Suggestion, Duration)>,
        operation: &OperationContext,
    ) {
        let Some(cache) = &self.storage_state.persistent_cache else {
            return;
        };
        let Some(epoch) = operation.persistent_epoch else {
            return;
        };
        if !operation.is_current() {
            return;
        }
        let now = timestamp_ms();
        let records = entries.filter_map(|(key, suggestion, ttl)| {
            // The host's reset ends a rate limit, so it is not kept across runs.
            if key.is_empty() || suggestion.status == SuggestionStatus::RateLimited {
                return None;
            }
            let value = serde_json::to_value(CachedSuggestion::from_suggestion(suggestion)).ok()?;
            let failed = suggestion.status == SuggestionStatus::Error;
            let deadline = now.saturating_add(ttl.as_millis().try_into().unwrap_or(u64::MAX));
            Some((
                key,
                PersistentRecord {
                    value,
                    attempted_at_ms: operation.attempted_at_ms,
                    succeeded_at_ms: (!failed).then_some(now),
                    expires_at_ms: deadline,
                    retry_at_ms: failed.then_some(deadline),
                    accessed_at_ms: now,
                },
            ))
        });
        let _ = cache.insert_many(epoch, records, now);
    }

    pub(crate) fn persistent_vulnerability_key(&self, dependency: &Dependency) -> String {
        let Some(cache) = &self.storage_state.persistent_cache else {
            return String::new();
        };
        let Ok(policy) = serde_json::to_vec(&self.config) else {
            return String::new();
        };
        let source =
            versionlens_providers::vulnerability_url(dependency.ecosystem).unwrap_or_default();
        let identity = serde_json::json!([
            dependency.ecosystem,
            dependency.name,
            dependency.requirement,
            dependency.hosted_name,
            dependency.hosted_url,
            dependency.group,
            persistent_canonical_identity(dependency.canonical_reference.as_ref()),
        ]);
        cache.partition_key(&[
            b"vulnerability-v1",
            source.as_bytes(),
            &policy,
            identity.to_string().as_bytes(),
        ])
    }

    pub(crate) fn load_persistent_vulnerability(
        &self,
        dependency: &Dependency,
    ) -> Option<(VulnerabilityCheck, Duration)> {
        if !supports_vulnerability_check(dependency) {
            return Some((
                VulnerabilityCheck::Unsupported,
                self.cache_ttl(dependency.ecosystem, None),
            ));
        }
        let now = timestamp_ms();
        let record = self
            .storage_state
            .persistent_cache
            .as_ref()?
            .get(&self.persistent_vulnerability_key(dependency), now)
            .ok()??;
        vulnerability_record(record, now)
    }

    pub(crate) fn store_persistent_vulnerability(
        &self,
        dependency: &Dependency,
        check: &VulnerabilityCheck,
        ttl: Duration,
        operation: &OperationContext,
    ) {
        if matches!(check, VulnerabilityCheck::Unsupported) {
            return;
        }
        self.store_persistent_outcome(
            self.persistent_vulnerability_key(dependency),
            PersistentOutcome {
                value: &CachedVulnerabilityCheck::from_check(check),
                failed: matches!(check, VulnerabilityCheck::Failed(_)),
            },
            ttl,
            operation,
        );
    }

    fn store_persistent_outcome<T: serde::Serialize>(
        &self,
        key: String,
        outcome: PersistentOutcome<'_, T>,
        ttl: Duration,
        operation: &OperationContext,
    ) {
        let Some(cache) = &self.storage_state.persistent_cache else {
            return;
        };
        let Some(epoch) = operation.persistent_epoch else {
            return;
        };
        if key.is_empty() || !operation.is_current() {
            return;
        }
        let Ok(value) = serde_json::to_value(outcome.value) else {
            return;
        };
        let now = timestamp_ms();
        let deadline = now.saturating_add(ttl.as_millis().try_into().unwrap_or(u64::MAX));
        let _ = cache.insert(
            epoch,
            key,
            PersistentRecord {
                value,
                attempted_at_ms: operation.attempted_at_ms,
                succeeded_at_ms: (!outcome.failed).then_some(now),
                expires_at_ms: deadline,
                retry_at_ms: outcome.failed.then_some(deadline),
                accessed_at_ms: now,
            },
            now,
        );
    }

    pub(crate) fn persistent_latest_key(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> Option<String> {
        let cache = self.storage_state.persistent_cache.as_ref()?;
        let policy = serde_json::to_vec(&self.config).ok()?;
        let source = format!("{context:?}");
        let dependency = cache::latest_cache_key(dependency);
        Some(cache.partition_key(&[
            b"latest-v1",
            dependency.as_str().as_bytes(),
            source.as_bytes(),
            &policy,
        ]))
    }

    pub(crate) fn load_persistent_latest(&self, key: &str) -> Option<(CachedLatest, Duration)> {
        let now = timestamp_ms();
        let record = self
            .storage_state
            .persistent_cache
            .as_ref()?
            .get(key, now)
            .ok()??;
        if record.expires_at_ms <= now || record.succeeded_at_ms.is_none() {
            return None;
        }
        let value = serde_json::from_value(record.value).ok()?;
        Some((value, Duration::from_millis(record.expires_at_ms - now)))
    }

    pub(crate) fn store_persistent_latest(
        &self,
        key: String,
        latest: &CachedLatest,
        ttl: Duration,
        operation: &OperationContext,
    ) {
        self.store_persistent_outcome(
            key,
            PersistentOutcome {
                value: latest,
                failed: false,
            },
            ttl,
            operation,
        );
    }
}

fn suggestion_record(
    mut record: PersistentRecord,
    dependency: &Dependency,
    now: u64,
) -> Option<(Suggestion, Duration)> {
    let cached: CachedSuggestion = serde_json::from_value(record.value.take()).ok()?;
    let ttl = outcome_ttl(&record, cached.status == SuggestionStatus::Error, now)?;
    Some((cached.into_suggestion(dependency), ttl))
}

fn vulnerability_record(
    mut record: PersistentRecord,
    now: u64,
) -> Option<(VulnerabilityCheck, Duration)> {
    let check: CachedVulnerabilityCheck = serde_json::from_value(record.value.take()).ok()?;
    let ttl = outcome_ttl(
        &record,
        matches!(check, CachedVulnerabilityCheck::Failed(_)),
        now,
    )?;
    Some((check.into_check(), ttl))
}

fn outcome_ttl(record: &PersistentRecord, failed: bool, now: u64) -> Option<Duration> {
    let valid = if failed {
        record.succeeded_at_ms.is_none() && record.retry_at_ms.is_some_and(|retry| retry > now)
    } else {
        record.succeeded_at_ms.is_some() && record.retry_at_ms.is_none()
    };
    (valid && record.expires_at_ms > now).then(|| Duration::from_millis(record.expires_at_ms - now))
}

fn persistent_canonical_identity(reference: Option<&CanonicalReference>) -> serde_json::Value {
    match reference {
        Some(CanonicalReference::GitHubActionSha { commit, tag, .. }) => {
            serde_json::json!({ "kind": "githubActionSha", "commit": commit, "tag": tag })
        }
        Some(reference) => serde_json::to_value(reference).unwrap_or(serde_json::Value::Null),
        None => serde_json::Value::Null,
    }
}

#[cfg(test)]
mod tests {
use crate::ProviderSettings;
use crate::cache::vulnerability_cache_key;
use crate::{ApplyCommandRequest, RegistryResponseInput, RegistryUrlConfig, VersionLensSession};
use std::time::Duration;
use versionlens_fetch::PersistentRecord;
use versionlens_model::{CanonicalReference, DocumentInput, Ecosystem::Dotnet, Ecosystem::Npm};
use versionlens_update::{Suggestion, SuggestionStatus};

fn session(directory: &std::path::Path, credential: &str) -> VersionLensSession {
    configured_session(directory, credential, "http://127.0.0.1:1", None)
}

fn configured_session(
    directory: &std::path::Path,
    credential: &str,
    registry_url: &str,
    cache_ttl_ms: Option<u64>,
) -> VersionLensSession {
    let mut config = crate::test_support::session_config(ProviderSettings::default(), false);
    config.providers.registry_urls = vec![RegistryUrlConfig {
        ecosystem: Npm,
        url: registry_url.to_owned(),
    }];
    if let Some(cache_ttl_ms) = cache_ttl_ms {
        config.cache_ttl_ms = cache_ttl_ms;
    }
    // The budget covers the whole check, including local file reads and
    // cache writes, so it must hold when every test runs at once. The
    // registry port refuses connections, so failures stay fast.
    config.http.timeout_ms = 10_000;
    config.http.auth_headers = vec![versionlens_fetch::HttpHeader {
        name: "Authorization".to_owned(),
        value: credential.to_owned(),
        url: Some("http://127.0.0.1:1".to_owned()),
    }];
    VersionLensSession::new(config)
        .with_persistent_cache(directory)
        .unwrap()
}

fn dotnet_session(directory: &std::path::Path) -> VersionLensSession {
    let mut config = crate::test_support::session_config(ProviderSettings::default(), false);
    config.providers.registry_urls = vec![RegistryUrlConfig {
        ecosystem: Dotnet,
        url: "http://127.0.0.1:1".to_owned(),
    }];
    // The budget covers the whole check, including local file reads and
    // cache writes, so it must hold when every test runs at once. The
    // registry port refuses connections, so failures stay fast.
    config.http.timeout_ms = 10_000;
    let session = VersionLensSession::new(config)
        .with_persistent_cache(directory)
        .unwrap();
    // Without this, NuGet sources come from `dotnet nuget list source` on
    // the test machine, which can reach the real nuget.org.
    *session
        .request_state
        .dotnet_registry_sources
        .lock()
        .unwrap() = Some(vec!["http://127.0.0.1:1".to_owned()]);
    session
}

fn input() -> DocumentInput {
    DocumentInput::new(
        "file:///workspace/package.json",
        "json",
        r#"{"dependencies":{"example":"1.0.0"}}"#,
        None,
    )
}

async fn populate(session: &VersionLensSession) {
    populate_input(session, input()).await;
}

async fn populate_input(session: &VersionLensSession, input: DocumentInput) {
    let output = session
        .resolve_document_with_responses(
            input,
            &[RegistryResponseInput::new(
                "example".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
            )],
        )
        .await;
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

fn failed_suggestion(dependency: versionlens_model::Dependency) -> Suggestion {
    Suggestion {
        dependency,
        latest: None,
        resolved: Some("registry unavailable".to_owned()),
        status: SuggestionStatus::Error,
        builds: vec![],
        choices: vec![],
    }
}

#[tokio::test]
async fn session_restart_reuses_checked_versions_without_a_registry_connection() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    populate(&session(&directory, "credential-a")).await;
    let restarted = session(&directory, "credential-a");
    let output = restarted.resolve_document(input()).await;
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
    let disk = std::fs::read_to_string(directory.join("cache.json")).unwrap();
    assert!(!disk.contains("credential-a"));
    assert!(!disk.contains("127.0.0.1"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn session_restart_preserves_dotnet_fixed_response_semantics() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = DocumentInput::new(
        "file:///coverage/app.csproj",
        "xml",
        "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
        None,
    );
    let first = session(&directory, "credential-a")
        .resolve_document_with_responses(
            input.clone(),
            &[RegistryResponseInput::new(
                "Foo",
                Dotnet,
                r#"{"versions":["1.0.0","2.0.0"]}"#,
            )],
        )
        .await;
    assert_eq!(first.suggestions[0].status, "fixed");
    assert_eq!(first.suggestions[0].latest.as_deref(), Some("1.0.0"));

    let restarted = session(&directory, "credential-a");
    let cached = restarted.resolve_document(input.clone()).await;
    assert_eq!(cached.suggestions[0].status, first.suggestions[0].status);
    assert_eq!(cached.suggestions[0].latest, first.suggestions[0].latest);
    assert!(cached.edits.is_empty());

    let analysis = restarted.analyze_document(input.clone());
    assert!(
        analysis
            .code_lenses
            .iter()
            .any(|lens| { lens.arguments().iter().any(|argument| argument == "2.0.0") })
    );

    let update = restarted
        .apply_command_with_selected_version(ApplyCommandRequest {
            input: input.clone(),
            command: Some("update"),
            dependency_name: Some("Foo"),
            selected_version: Some("2.0.0"),
            responses: &[],
        })
        .await;
    assert_eq!(update.edits.len(), 1);
    assert_eq!(update.edits[0].new_text, "2.0.0");
    let requirement_start = u32::try_from(input.text.find("1.0.0").unwrap()).unwrap();
    assert_eq!(update.edits[0].range.start.line, 0);
    assert_eq!(update.edits[0].range.start.character, requirement_start);
    assert_eq!(update.edits[0].range.end.character, requirement_start + 5);
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn fixed_pin_revalidates_cached_latest_record_without_proof_field() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = DocumentInput::new(
        "file:///coverage/app.csproj",
        "xml",
        "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
        None,
    );
    let first = dotnet_session(&directory);
    let dependency = first.dependencies(&input).remove(0);
    let manifest_kind = first.classify_document(&input);
    let context = first.registry_context(&input, manifest_kind);
    let key = first.persistent_latest_key(&dependency, &context).unwrap();
    let cache = first.storage_state.persistent_cache.as_ref().unwrap();
    let epoch = cache.epoch().unwrap();
    let now = super::timestamp_ms();
    cache
        .insert(
            epoch,
            key,
            PersistentRecord {
                value: serde_json::json!({
                    "latest": "2.0.0",
                    "builds": [],
                    "choices": []
                }),
                attempted_at_ms: now,
                succeeded_at_ms: Some(now),
                expires_at_ms: now + 60_000,
                retry_at_ms: None,
                accessed_at_ms: now,
            },
            now,
        )
        .unwrap();
    drop(first);

    let revalidated = dotnet_session(&directory).resolve_document(input).await;
    assert_eq!(revalidated.suggestions[0].status, "error");
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn restart_restores_semantic_freshness_with_current_document_ranges() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let initial = DocumentInput::new(
        "file:///workspace/package.json5",
        "json5",
        "{'dependencies':{'example':'1.0.0'}}",
        None,
    );
    populate_input(&session(&directory, "credential-a"), initial).await;

    let moved = DocumentInput::new(
        "file:///workspace/package.json5",
        "json5",
        "{\n  \"dependencies\": {\n    \"example\": \"1.0.0\"\n  }\n}",
        None,
    );
    let restarted = session(&directory, "credential-a");
    assert!(restarted.document_is_fresh(&moved));
    let analyzed = restarted.analyze_document(moved);
    assert!(!analyzed.code_lenses.is_empty());
    assert!(
        analyzed
            .code_lenses
            .iter()
            .all(|lens| lens.range.start.line == 2)
    );

    let disk = std::fs::read_to_string(directory.join("cache.json")).unwrap();
    assert!(!disk.contains("credential-a"));
    assert!(!disk.contains("requirementRange"));
    assert!(!disk.contains("requirementPrefix"));
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn failed_outcomes_survive_restart_only_until_their_retry_deadline() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let failed = first.resolve_document(input()).await;
    assert_eq!(failed.suggestions[0].status, "error");

    let restarted = session(&directory, "credential-a");
    assert!(restarted.document_is_fresh(&input()));

    let dependency = restarted.dependencies(&input()).remove(0);
    let kind = restarted.classify_document(&input());
    let context = restarted.registry_context(&input(), kind);
    let scope = restarted.document_cache_scope(&context, &input());
    let operation = restarted.operation_context();
    let key = restarted.persistent_suggestion_key(&dependency, &scope);
    restarted.store_persistent_suggestions(
        std::iter::once((key, &failed_suggestion(dependency), Duration::ZERO)),
        &operation,
    );

    let expired = session(&directory, "credential-a");
    assert!(!expired.document_is_fresh(&input()));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn local_vulnerability_status_is_available_after_restart_without_disk_records() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let local = DocumentInput::new(
        "file:///workspace/package.json",
        "json",
        r#"{"dependencies":{"example":"file:../example"}}"#,
        None,
    );
    let dependency = first.dependencies(&local).remove(0);
    let operation = first.operation_context();
    first.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Unsupported,
        Duration::from_secs(60),
        &operation,
    );
    let cache = first.storage_state.persistent_cache.as_ref().unwrap();
    assert!(
        cache
            .get(
                &first.persistent_vulnerability_key(&dependency),
                super::timestamp_ms()
            )
            .unwrap()
            .is_none()
    );
    let restarted = session(&directory, "credential-a");
    assert!(matches!(
        restarted.load_persistent_vulnerability(&dependency),
        Some((crate::vulnerability::VulnerabilityCheck::Unsupported, ttl)) if !ttl.is_zero()
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn restarted_session_rechecks_changed_local_action_manifests() {
    let directory =
        versionlens_test_support::temporary_directory("versionlens-local-action").unwrap();
    let root = directory.join("workspace");
    let action = root.join("tool/action.yml");
    std::fs::create_dir_all(action.parent().unwrap()).unwrap();
    std::fs::write(&action, "runs: {using: composite, steps: []}\n").unwrap();
    let input = DocumentInput::new(
        crate::workspace_file_uri(&root.join(".github/workflows/ci.yml")).unwrap(),
        "yaml",
        "jobs: {check: {steps: [{uses: './tool'}]}}\n",
        Some(root.to_string_lossy().into_owned()),
    );
    let storage = directory.join("cache");
    let first = session(&storage, "credential-a")
        .resolve_document(input.clone())
        .await;
    assert_eq!(first.suggestions[0].status, "fixed");

    std::fs::write(&action, "runs: {}\n").unwrap();
    let restarted = session(&storage, "credential-a");
    assert!(!restarted.document_is_fresh(&input));
    let output = restarted.resolve_document(input).await;
    assert_eq!(output.suggestions[0].status, "error");
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn vulnerability_empty_and_failed_outcomes_remain_distinct_after_restart() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let mut first = session(&directory, "credential-a");
    first.config.show_vulnerabilities = true;
    let dependency = first.dependencies(&input()).remove(0);
    let operation = first.operation_context();
    first.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Checked(vec![]),
        Duration::from_secs(60),
        &operation,
    );
    let mut restarted = session(&directory, "credential-a");
    restarted.config.show_vulnerabilities = true;
    restarted.analyze_document(input());
    assert!(matches!(
        restarted.vulnerability_cache().get(&vulnerability_cache_key(&dependency)),
        Some(crate::vulnerability::VulnerabilityCheck::Checked(advisories))
            if advisories.is_empty()
    ));

    let operation = restarted.operation_context();
    restarted.store_persistent_vulnerability(
        &dependency,
        &crate::vulnerability::VulnerabilityCheck::Failed("service unavailable".to_owned()),
        Duration::from_secs(60),
        &operation,
    );
    let mut failed = session(&directory, "credential-a");
    failed.config.show_vulnerabilities = true;
    failed.analyze_document(input());
    assert!(matches!(
        failed.vulnerability_cache().get(&vulnerability_cache_key(&dependency)),
        Some(crate::vulnerability::VulnerabilityCheck::Failed(message))
            if message == "service unavailable"
    ));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_semantic_keys_ignore_formatting_but_isolate_canonical_identity() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let original = first.dependencies(&input()).remove(0);
    let kind = first.classify_document(&input());
    let context = first.registry_context(&input(), kind);
    let scope = first.document_cache_scope(&context, &input());
    let original_key = first.persistent_suggestion_key(&original, &scope);

    let mut formatted = original;
    formatted.range.start.line = 99;
    formatted.requirement_range.start.character = 42;
    formatted.requirement_prefix = "'".to_owned();
    formatted.requirement_suffix = "'".to_owned();
    assert_eq!(
        original_key,
        first.persistent_suggestion_key(&formatted, &scope)
    );

    formatted.canonical_reference = Some(CanonicalReference::GitHubActionTag {
        tag: "v1".to_owned(),
    });
    assert_ne!(
        original_key,
        first.persistent_suggestion_key(&formatted, &scope)
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_semantic_keys_isolate_authentication_source_and_policy() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let input = input();
    let key = |session: &VersionLensSession| {
        let dependency = session.dependencies(&input).remove(0);
        let kind = session.classify_document(&input);
        let context = session.registry_context(&input, kind);
        let scope = session.document_cache_scope(&context, &input);
        session.persistent_suggestion_key(&dependency, &scope)
    };
    let baseline = configured_session(&directory, "credential-a", "http://127.0.0.1:1", None);
    let different_auth = configured_session(&directory, "credential-b", "http://127.0.0.1:1", None);
    let different_source =
        configured_session(&directory, "credential-a", "http://127.0.0.1:2", None);
    let different_policy = configured_session(
        &directory,
        "credential-a",
        "http://127.0.0.1:1",
        Some(123_456),
    );
    let baseline = key(&baseline);
    assert_ne!(baseline, key(&different_auth));
    assert_ne!(baseline, key(&different_source));
    assert_ne!(baseline, key(&different_policy));
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn persistent_outcomes_record_attempt_success_expiry_and_retry_separately() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let session = session(&directory, "credential-a");
    let dependency = session.dependencies(&input()).remove(0);
    let kind = session.classify_document(&input());
    let context = session.registry_context(&input(), kind);
    let scope = session.document_cache_scope(&context, &input());
    let key = session.persistent_suggestion_key(&dependency, &scope);
    let operation = session.operation_context();
    session.store_persistent_suggestions(
        std::iter::once((
            key.clone(),
            &failed_suggestion(dependency.clone()),
            Duration::from_secs(60),
        )),
        &operation,
    );
    let cache = session.storage_state.persistent_cache.as_ref().unwrap();
    let failure = cache.get(&key, super::timestamp_ms()).unwrap().unwrap();
    assert_eq!(failure.attempted_at_ms, operation.attempted_at_ms);
    assert_eq!(failure.succeeded_at_ms, None);
    assert_eq!(failure.retry_at_ms, Some(failure.expires_at_ms));

    let success = Suggestion {
        dependency,
        latest: Some("2.0.0".to_owned()),
        resolved: None,
        status: SuggestionStatus::UpdateAvailable,
        builds: vec![],
        choices: vec![],
    };
    let operation = session.operation_context();
    session.store_persistent_suggestions(
        std::iter::once((key.clone(), &success, Duration::from_secs(60))),
        &operation,
    );
    let success = cache.get(&key, super::timestamp_ms()).unwrap().unwrap();
    assert_eq!(success.attempted_at_ms, operation.attempted_at_ms);
    assert!(success.succeeded_at_ms.is_some());
    assert_eq!(success.retry_at_ms, None);
    assert!(success.expires_at_ms > success.succeeded_at_ms.unwrap());
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn clear_epoch_rejects_a_late_semantic_outcome_write() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let dependency = first.dependencies(&input()).remove(0);
    let kind = first.classify_document(&input());
    let context = first.registry_context(&input(), kind);
    let scope = first.document_cache_scope(&context, &input());
    let key = first.persistent_suggestion_key(&dependency, &scope);
    let operation = first.operation_context();
    first.try_clear_cache().unwrap();
    first.store_persistent_suggestions(
        std::iter::once((key, &failed_suggestion(dependency), Duration::from_secs(60))),
        &operation,
    );

    assert!(!session(&directory, "credential-a").document_is_fresh(&input()));
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn authentication_partitions_require_independent_checks() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    populate(&session(&directory, "credential-a")).await;
    let other = session(&directory, "credential-b");
    assert_check_failed(&other).await;
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn clearing_persistent_results_requires_a_new_successful_check() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    populate(&first).await;
    first.clear_cache();
    let restarted = session(&directory, "credential-a");
    assert_check_failed(&restarted).await;
    std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn a_clear_from_another_session_invalidates_memory_results() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    populate(&first).await;
    assert!(!first.analyze_document(input()).code_lenses.is_empty());
    session(&directory, "credential-a")
        .try_clear_cache()
        .unwrap();
    assert!(first.analyze_document(input()).code_lenses.is_empty());
    assert_check_failed(&first).await;
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn a_disk_generation_change_invalidates_pending_publication() {
    let directory = versionlens_test_support::temporary_directory("versionlens-storage").unwrap();
    let first = session(&directory, "credential-a");
    let operation = first.operation_context();
    assert!(operation.can_publish());
    session(&directory, "credential-a")
        .try_clear_cache()
        .unwrap();
    assert!(!operation.can_publish());
    std::fs::remove_dir_all(directory).unwrap();
}

async fn assert_check_failed(session: &VersionLensSession) {
    let output = session.resolve_document(input()).await;
    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "error");
}
}

mod hydration;
