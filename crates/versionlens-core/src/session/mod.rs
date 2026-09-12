use std::collections::{HashMap, hash_map::RandomState};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::vulnerability::VulnerabilityCheck;
use versionlens_cache::{CacheKey, MemoryCache};
use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind, TextEdit};
use versionlens_suggestions::Suggestion;

use crate::concurrency::Priority;
use crate::config::SessionConfig;
use crate::contract;
use crate::contract::{
    AuthorizationRequestPayload, ResolveDocumentOutput, ResolveDocumentOutputParts,
    resolve_document_output,
};
use crate::suggestion::into_suggestion_payloads;
use cache::CachedLatest;

mod cache;
mod checking;
#[cfg(test)]
#[path = "checking_matrix/tests.rs"]
mod checking_matrix;
mod classify;
mod commands;
mod dependencies;
mod discovery;
mod documents;
pub(crate) mod operation;
mod presentation;
mod resolution;
mod storage;
mod tasks;
mod workspace;

pub use checking::{
    WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking, WorkspaceCheckingOptions,
};
pub use commands::ApplyCommandRequest;
pub use discovery::{
    WorkspaceDiscovery, WorkspaceDiscoveryCancellation, WorkspaceDiscoveryFailure,
    WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryIoOperation, WorkspaceDiscoveryLimits,
    WorkspaceDiscoveryOptions, WorkspaceProviderExclusion, workspace_exclusion_matches,
    workspace_exclusion_path,
};
pub use tasks::{SessionTask, TaskCancellation};

#[derive(Debug, Clone)]
pub struct VersionLensSession {
    pub(crate) config: SessionConfig,
    pub(crate) storage_state: SessionStorage,
    pub(crate) request_state: SessionRequests,
    pub(crate) result_state: SessionResults,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionStorage {
    pub(crate) persistent_cache: Option<Arc<versionlens_cache::PersistentCache>>,
    pub(crate) observed_persistent_epoch: Arc<Mutex<Option<u64>>>,
    pub(crate) cache_epoch: Arc<AtomicU64>,
    pub(crate) task_epoch: Option<u64>,
    pub(crate) task_cancellation: Option<Arc<std::sync::atomic::AtomicBool>>,
    pub(crate) task_priority: Priority,
    pub(crate) workspace: Arc<Mutex<workspace::WorkspaceState>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionRequests {
    pub(crate) request_body_cache: Arc<Mutex<MemoryCache<String>>>,
    pub(crate) request_locks: Arc<Mutex<HashMap<CacheKey, Weak<Mutex<()>>>>>,
    pub(crate) request_context_hashers: [RandomState; 2],
    pub(crate) dotnet_registry_sources: Arc<Mutex<Option<Vec<String>>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionResults {
    pub(crate) parsed_dependencies: Arc<Mutex<MemoryCache<Vec<Dependency>>>>,
    pub(crate) latest: Arc<Mutex<MemoryCache<CachedLatest>>>,
    pub(crate) suggestions: Arc<Mutex<MemoryCache<Suggestion>>>,
    pub(crate) vulnerabilities: Arc<Mutex<MemoryCache<VulnerabilityCheck>>>,
}

impl VersionLensSession {
    pub fn cancel_pending_resolutions(&self) {
        self.storage_state
            .cache_epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }

    pub fn new(config: SessionConfig) -> Self {
        version_lens_session(config)
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

pub fn version_lens_session(config: SessionConfig) -> VersionLensSession {
    let config = SessionConfig {
        suggestion_indicators: config
            .suggestion_indicators
            .with_standard_indicators_for_blanks(),
        ..config
    };
    let cache_ttl = crate::duration_from_millis(config.cache_ttl_ms);
    VersionLensSession {
        config,
        storage_state: SessionStorage {
            persistent_cache: None,
            observed_persistent_epoch: Arc::new(Mutex::new(None)),
            cache_epoch: Arc::new(AtomicU64::new(0)),
            task_cancellation: None,
            task_epoch: None,
            task_priority: Priority::Interactive,
            workspace: Arc::new(Mutex::new(workspace::WorkspaceState::default())),
        },
        request_state: SessionRequests {
            request_body_cache: Arc::new(crate::mutex(
                crate::memory_cache(cache_ttl)
                    .with_byte_capacity(32 * 1024 * 1024, String::capacity),
            )),
            request_locks: Arc::new(crate::mutex(crate::default())),
            request_context_hashers: [<RandomState>::new(), <RandomState>::new()],
            dotnet_registry_sources: Arc::new(crate::mutex(None)),
        },
        result_state: SessionResults {
            parsed_dependencies: Arc::new(crate::mutex(
                crate::memory_cache(Duration::from_secs(300))
                    .with_capacity(64)
                    .with_byte_capacity(16 * 1024 * 1024, cache::weights::dependencies),
            )),
            latest: Arc::new(crate::mutex(
                crate::memory_cache(cache_ttl)
                    .with_byte_capacity(16 * 1024 * 1024, cache::weights::latest),
            )),
            suggestions: Arc::new(crate::mutex(
                crate::memory_cache(cache_ttl)
                    .with_byte_capacity(8 * 1024 * 1024, cache::weights::suggestion),
            )),
            vulnerabilities: Arc::new(crate::mutex(
                crate::memory_cache(cache_ttl)
                    .with_byte_capacity(8 * 1024 * 1024, cache::weights::vulnerability),
            )),
        },
    }
}

fn cancelled_resolution(suggestions: Vec<Suggestion>) -> ResolveDocumentOutput {
    let suggestions = suggestions
        .into_iter()
        .map(|suggestion| {
            versionlens_suggestions::error(suggestion.dependency, "checking cancelled".to_owned())
        })
        .collect();
    finish_resolve_output(suggestions, resolve_output_parts(vec![], 0, vec![], 0))
}

pub(crate) fn finish_resolve_output(
    suggestions: Vec<Suggestion>,
    mut parts: ResolveDocumentOutputParts,
) -> ResolveDocumentOutput {
    parts.suggestions = into_suggestion_payloads(suggestions);
    resolve_document_output(parts)
}

pub(crate) fn resolve_output_parts(
    edits: Vec<TextEdit>,
    authorization_required_count: u32,
    authorization_required_requests: Vec<AuthorizationRequestPayload>,
    vulnerable_update_count: u32,
) -> ResolveDocumentOutputParts {
    ResolveDocumentOutputParts {
        suggestions: Vec::new(),
        edits,
        authorization_required_count,
        authorization_required_requests,
        vulnerable_update_count,
        vulnerable_update_package: None,
        vulnerable_update_version: None,
        edit_plan: None,
    }
}

pub(crate) fn resolve_output_parts_with_plan(
    input: &DocumentInput,
    edits: Vec<TextEdit>,
    authorization_required_count: u32,
    authorization_required_requests: Vec<AuthorizationRequestPayload>,
    vulnerable_update_count: u32,
) -> ResolveDocumentOutputParts {
    let mut parts = resolve_output_parts(
        edits.clone(),
        authorization_required_count,
        authorization_required_requests,
        vulnerable_update_count,
    );
    parts.edit_plan = contract::document_edit_plan(input, &edits);
    if parts.edit_plan.is_none() {
        // A raw edit without its snapshot is unsafe; fail closed.
        parts.edits.clear();
    }
    parts
}
