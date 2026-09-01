use std::collections::{HashMap, hash_map::RandomState};
use std::sync::{Mutex, Weak};

use versionlens_cache::{CacheKey, MemoryCache};
use versionlens_model::{DocumentInput, Ecosystem, ManifestKind, TextEdit};
use versionlens_providers::VulnerabilityAdvisory;
use versionlens_suggestions::Suggestion;

use crate::config::SessionConfig;
use crate::contract;
use crate::contract::{
    AuthorizationRequestPayload, ResolveDocumentOutput, ResolveDocumentOutputParts,
    resolve_document_output,
};
use crate::suggestion::into_suggestion_payloads;
use cache::CachedLatest;

mod cache;
mod classify;
mod commands;
mod dependencies;
mod documents;
pub(crate) mod operation;
mod presentation;
mod resolution;

pub use commands::ApplyCommandRequest;

#[derive(Debug)]
pub struct VersionLensSession {
    pub(crate) config: SessionConfig,
    pub(crate) latest_cache: Mutex<MemoryCache<CachedLatest>>,
    pub(crate) request_body_cache: Mutex<MemoryCache<String>>,
    pub(crate) request_locks: Mutex<HashMap<CacheKey, Weak<Mutex<()>>>>,
    pub(crate) request_context_hashers: [RandomState; 2],
    pub(crate) suggestion_cache: Mutex<MemoryCache<Suggestion>>,
    pub(crate) vulnerability_cache: Mutex<MemoryCache<Vec<VulnerabilityAdvisory>>>,
    pub(crate) dotnet_registry_sources: Mutex<Option<Vec<String>>>,
}

impl VersionLensSession {
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
        latest_cache: crate::mutex(crate::memory_cache(cache_ttl)),
        request_body_cache: crate::mutex(crate::memory_cache(cache_ttl)),
        request_locks: crate::mutex(crate::default()),
        request_context_hashers: [<RandomState>::new(), <RandomState>::new()],
        suggestion_cache: crate::mutex(crate::memory_cache(cache_ttl)),
        vulnerability_cache: crate::mutex(crate::memory_cache(cache_ttl)),
        dotnet_registry_sources: crate::mutex(None),
    }
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
