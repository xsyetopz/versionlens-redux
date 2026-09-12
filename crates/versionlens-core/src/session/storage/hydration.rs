use std::sync::atomic::Ordering;

use versionlens_cache::{CacheKey, PersistentRecord};
use versionlens_model::{Dependency, DocumentInput};

use super::{suggestion_record, timestamp_ms, vulnerability_record};
use crate::VersionLensSession;
use crate::cache::{suggestion_cache_key, vulnerability_cache_key};
use crate::vulnerability::{VulnerabilityCheck, supports_vulnerability_check};

enum OutcomeKind {
    Suggestion,
    Vulnerability,
}

struct HydrationEntry<'a> {
    dependency: &'a Dependency,
    memory_key: CacheKey,
    persistent_key: String,
    kind: OutcomeKind,
}

impl VersionLensSession {
    pub(super) fn hydrate_persistent_outcomes(&self, input: &DocumentInput, scope: &str) {
        let Some(cache) = &self.storage_state.persistent_cache else {
            return;
        };
        let epoch = self.storage_state.cache_epoch.load(Ordering::Acquire);
        let dependencies = self.dependencies(input);
        let entries = self.hydration_entries(&dependencies, scope, epoch);
        if entries.is_empty() {
            return;
        }
        let now = timestamp_ms();
        let Ok(records) = cache.get_many(
            entries.iter().map(|entry| entry.persistent_key.as_str()),
            now,
        ) else {
            return;
        };
        self.synchronize_persistent_cache();
        for (entry, record) in entries.into_iter().zip(records) {
            if let Some(record) = record {
                entry.restore(self, record, epoch, now);
            }
        }
    }

    fn hydration_entries<'a>(
        &self,
        dependencies: &'a [Dependency],
        scope: &str,
        epoch: u64,
    ) -> Vec<HydrationEntry<'a>> {
        let mut entries = Vec::new();
        for dependency in dependencies {
            let memory_key = suggestion_cache_key(dependency, scope);
            if self.suggestion_cache().get(&memory_key).is_none() {
                let persistent_key = self.persistent_suggestion_key(dependency, scope);
                if !persistent_key.is_empty() {
                    entries.push(HydrationEntry {
                        dependency,
                        memory_key,
                        persistent_key,
                        kind: OutcomeKind::Suggestion,
                    });
                }
            }
            if !self.config.show_vulnerabilities {
                continue;
            }
            let memory_key = vulnerability_cache_key(dependency);
            let mut memory = self.vulnerability_cache();
            if memory.get(&memory_key).is_some() {
                continue;
            }
            if supports_vulnerability_check(dependency) {
                drop(memory);
                entries.push(HydrationEntry {
                    dependency,
                    memory_key,
                    persistent_key: self.persistent_vulnerability_key(dependency),
                    kind: OutcomeKind::Vulnerability,
                });
            } else {
                if self.storage_state.cache_epoch.load(Ordering::Acquire) == epoch {
                    memory.insert_with_ttl(
                        memory_key,
                        VulnerabilityCheck::Unsupported,
                        self.cache_ttl(dependency.ecosystem, None),
                    );
                }
                drop(memory);
            }
        }
        entries
    }
}

impl HydrationEntry<'_> {
    fn restore(self, session: &VersionLensSession, record: PersistentRecord, epoch: u64, now: u64) {
        match self.kind {
            OutcomeKind::Suggestion => {
                if let Some((suggestion, ttl)) = suggestion_record(record, self.dependency, now) {
                    let mut cache = session.suggestion_cache();
                    if session.storage_state.cache_epoch.load(Ordering::Acquire) == epoch {
                        cache.insert_with_ttl(self.memory_key, suggestion, ttl);
                    }
                }
            }
            OutcomeKind::Vulnerability => {
                if let Some((check, ttl)) = vulnerability_record(record, now) {
                    let mut cache = session.vulnerability_cache();
                    if session.storage_state.cache_epoch.load(Ordering::Acquire) == epoch {
                        cache.insert_with_ttl(self.memory_key, check, ttl);
                    }
                }
            }
        }
    }
}
