use std::path::PathBuf;
use std::sync::Arc;

use versionlens_core::{
    VersionLensSession, WorkspaceChecking, WorkspaceCheckingOptions, WorkspaceDiscoveryOptions,
    WorkspaceProviderExclusion,
};
use versionlens_model::DocumentInput;

use crate::binding::{
    NativeDocumentInput, NativeWorkspaceCheckEvent, NativeWorkspaceCheckingInput,
};

mod events;
use events::EventQueue;

#[derive(Clone, PartialEq, Eq)]
struct Snapshot {
    roots: Vec<PathBuf>,
    exclusions: Vec<String>,
    provider_exclusions: Vec<WorkspaceProviderExclusion>,
    documents: Vec<DocumentInput>,
}

impl Snapshot {
    fn from_input(input: NativeWorkspaceCheckingInput) -> Result<Self, String> {
        let provider_exclusions = input
            .provider_exclusions
            .unwrap_or_default()
            .into_iter()
            .map(|exclusion| {
                let ecosystem = versionlens_model::ecosystem_from_config_name(&exclusion.ecosystem)
                    .ok_or_else(|| {
                        format!(
                            "invalid discovery exclusion ecosystem: {}",
                            exclusion.ecosystem
                        )
                    })?;
                Ok(WorkspaceProviderExclusion {
                    ecosystem,
                    patterns: exclusion.patterns,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let roots = input
            .roots
            .into_iter()
            .map(|root| {
                versionlens_core::workspace_path(&root)
                    .filter(|path| path.is_absolute())
                    .ok_or_else(|| format!("invalid workspace root: {root}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let documents = input
            .documents
            .into_iter()
            .map(NativeDocumentInput::into_core)
            .collect::<Vec<_>>();
        let bytes = documents.iter().fold(0_usize, |total, input| {
            total
                .saturating_add(input.text.len())
                .saturating_add(input.uri.len())
                .saturating_add(input.language_id.len())
                .saturating_add(input.workspace_root.as_ref().map_or(0, String::len))
        });
        if bytes > 128 * 1024 * 1024 {
            return Err("workspace documents exceed the 128 MiB input limit".to_owned());
        }
        Ok(Self {
            roots,
            exclusions: input.exclusions,
            provider_exclusions,
            documents,
        })
    }

    fn options(&self) -> WorkspaceCheckingOptions {
        let mut options = WorkspaceDiscoveryOptions::new(self.roots.clone());
        options.exclusions = self.exclusions.clone();
        options.provider_exclusions = self.provider_exclusions.clone();
        options.overlays = self.documents.clone();
        WorkspaceCheckingOptions::new(options)
    }
}

#[derive(Default)]
pub(super) struct WorkspaceClient {
    handle: Option<WorkspaceChecking>,
    snapshot: Option<Snapshot>,
    events: Arc<EventQueue>,
}

impl WorkspaceClient {
    pub(super) fn check(
        &mut self,
        session: &VersionLensSession,
        input: NativeWorkspaceCheckingInput,
    ) -> Result<(String, bool), String> {
        let snapshot = Snapshot::from_input(input)?;
        if self.snapshot.as_ref() == Some(&snapshot)
            && let Some(generation) = self.generation()
        {
            return Ok((generation, false));
        }
        let generation = self.install(session, snapshot)?;
        Ok((generation, true))
    }

    fn install(
        &mut self,
        session: &VersionLensSession,
        snapshot: Snapshot,
    ) -> Result<String, String> {
        let options = snapshot.options();
        let generation = if let Some(handle) = &self.handle {
            Self::replace(handle, &self.events, options)?
        } else {
            self.events = Arc::new(EventQueue::default());
            let events = Arc::clone(&self.events);
            let handle = session
                .start_workspace_checking(options, move |event| events.push(event))
                .map_err(|error| error.to_string())?;
            let generation = handle.generation();
            self.handle = Some(handle);
            generation
        };
        self.snapshot = Some(snapshot);
        Ok(generation.to_string())
    }

    fn replace(
        handle: &WorkspaceChecking,
        events: &EventQueue,
        options: WorkspaceCheckingOptions,
    ) -> Result<u64, String> {
        let generation = handle
            .generation()
            .checked_add(1)
            .ok_or("workspace generation limit reached")?;
        events.reset(generation);
        handle.replace(options).map_err(str::to_owned)
    }

    pub(super) fn documents(
        &mut self,
        session: &VersionLensSession,
        documents: Vec<DocumentInput>,
    ) -> Result<bool, String> {
        let Some(snapshot) = &self.snapshot else {
            return Ok(false);
        };
        if snapshot.documents == documents {
            return Ok(false);
        }
        let mut snapshot = snapshot.clone();
        snapshot.documents = documents;
        self.install(session, snapshot)?;
        Ok(true)
    }

    pub(super) fn invalidate(&self) -> Result<(), String> {
        if let (Some(handle), Some(snapshot)) = (&self.handle, &self.snapshot) {
            Self::replace(handle, &self.events, snapshot.options())?;
        }
        Ok(())
    }

    pub(super) fn take(&self) -> Vec<NativeWorkspaceCheckEvent> {
        self.events.take()
    }

    pub(super) fn generation(&self) -> Option<String> {
        self.handle
            .as_ref()
            .map(|handle| handle.generation().to_string())
    }

    pub(super) fn close(&mut self) {
        self.events.close();
        self.handle = None;
        self.snapshot = None;
    }
}

impl Drop for WorkspaceClient {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests;
