use std::path::PathBuf;

use versionlens_model::DocumentInput;

use super::DiscoveryStep;
use super::support::verify_overlay_containment;
use super::{WorkspaceDiscovery, WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind};

impl WorkspaceDiscovery {
    pub(super) fn next_overlay(&mut self) -> DiscoveryStep {
        while let Some(input) = self.traversal.overlays.pop_front() {
            if let Some(cancelled) = self.cancelled() {
                return Some(cancelled);
            }
            if let Err(failure) = self.visit_entry(PathBuf::from(&input.uri)) {
                return Some(Err(failure));
            }
            match self.prepare_overlay(input) {
                Ok(Some((input, path))) => return Some(self.accept(input, path)),
                Ok(None) => {}
                Err(failure) => return Some(Err(failure)),
            }
        }
        self.traversal.overlays_complete = true;
        None
    }

    fn prepare_overlay(
        &mut self,
        mut input: DocumentInput,
    ) -> Result<Option<(DocumentInput, PathBuf)>, WorkspaceDiscoveryFailure> {
        let path = crate::workspace_path(&input.uri)
            .filter(|path| path.is_absolute())
            .ok_or_else(|| {
                WorkspaceDiscoveryFailure::at(
                    PathBuf::from(&input.uri),
                    WorkspaceDiscoveryFailureKind::InvalidOverlayUri,
                )
            })?;
        let path = if self.overlay_only {
            path
        } else {
            let (root_index, path) = self.root_for_path(&path).ok_or_else(|| {
                WorkspaceDiscoveryFailure::at(
                    path.clone(),
                    WorkspaceDiscoveryFailureKind::OutsideWorkspace,
                )
            })?;
            let root = &self.roots[root_index];
            verify_overlay_containment(&path, &root.path)?;
            input.uri = crate::workspace_file_uri(&path).ok_or_else(|| {
                WorkspaceDiscoveryFailure::at(
                    path.clone(),
                    WorkspaceDiscoveryFailureKind::InvalidOverlayUri,
                )
            })?;
            input.workspace_root = Some(root.uri.clone());
            path
        };
        if !self.traversal.seen_paths.insert(path.clone()) || !self.supported(&input, false) {
            return Ok(None);
        }
        let size = u64::try_from(input.text.len()).unwrap_or(u64::MAX);
        if size > self.limits.max_file_size {
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::FileTooLarge {
                    size,
                    limit: self.limits.max_file_size,
                },
            ));
        }
        Ok(Some((input, path)))
    }
}

#[cfg(test)]
mod tests;
