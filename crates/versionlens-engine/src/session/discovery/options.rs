use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use versionlens_model::{DocumentInput, Ecosystem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceDiscoveryLimits {
    pub files: usize,
    pub visited_entries: usize,
    pub depth: usize,
    pub file_size: u64,
}

impl Default for WorkspaceDiscoveryLimits {
    fn default() -> Self {
        Self {
            files: 10_000,
            visited_entries: 250_000,
            depth: 64,
            file_size: 4 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceDiscoveryCancellation(Arc<AtomicBool>);

impl WorkspaceDiscoveryCancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceDiscoveryOptions {
    /// Empty roots check only the supplied overlays, without reading their parent directories.
    pub roots: Vec<PathBuf>,
    pub exclusions: Vec<String>,
    pub provider_exclusions: Vec<WorkspaceProviderExclusion>,
    pub overlays: Vec<DocumentInput>,
    pub limits: WorkspaceDiscoveryLimits,
    pub cancellation: WorkspaceDiscoveryCancellation,
}

impl WorkspaceDiscoveryOptions {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        Self {
            roots,
            exclusions: Vec::new(),
            provider_exclusions: Vec::new(),
            overlays: Vec::new(),
            limits: WorkspaceDiscoveryLimits::default(),
            cancellation: WorkspaceDiscoveryCancellation::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProviderExclusion {
    pub ecosystem: Ecosystem,
    pub patterns: Vec<String>,
}
