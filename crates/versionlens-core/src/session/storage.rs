use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use versionlens_cache::{PersistentCache, PersistentRecord};
use versionlens_model::{CanonicalReference, Dependency};
use versionlens_suggestions::{Suggestion, SuggestionStatus, UpdateChoice};

use super::cache::CachedLatest;
use super::operation::{OperationContext, timestamp_ms};
use crate::registry::RegistryContext;
use crate::vulnerability::{VulnerabilityCheck, supports_vulnerability_check};
use crate::{VersionLensSession, cache, project};

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
    pub fn with_application_cache(self) -> std::io::Result<Self> {
        self.with_persistent_cache(versionlens_cache::application_cache_directory()?)
    }

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
            .unwrap_or_else(crate::recover_poison);
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
    ) -> String {
        let workspace = self.workspace_graph(input);
        let scope = serde_json::json!([
            self.cache_scope(context),
            input.uri,
            input.workspace_root,
            workspace
                .fingerprint
                .as_ref()
                .map(versionlens_cache::CacheKey::as_str),
        ])
        .to_string();
        self.hydrate_persistent_outcomes(input, &scope);
        scope
    }

    pub(crate) fn persistent_suggestion_key(&self, dependency: &Dependency, scope: &str) -> String {
        if project::is_project_version_dependency(dependency)
            || matches!(
                dependency.canonical_reference,
                Some(CanonicalReference::GitHubActionLocal { .. })
            )
        {
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
            scope.as_bytes(),
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
            if key.is_empty() {
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
mod tests;

mod hydration;
