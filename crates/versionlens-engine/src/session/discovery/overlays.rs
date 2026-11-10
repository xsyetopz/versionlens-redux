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
        // An open document outside every root is checked as it is, the
        // same as when it is opened after discovery.
        let root = (!self.overlay_only)
            .then(|| self.root_for_path(&path))
            .flatten();
        let path = if let Some((root_index, path)) = root {
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
        } else {
            path
        };
        if !self.traversal.seen_paths.insert(path.clone()) || !self.supported(&input, false) {
            return Ok(None);
        }
        let size = u64::try_from(input.text.len()).unwrap_or(u64::MAX);
        if size > self.limits.file_size {
            return Err(WorkspaceDiscoveryFailure::file_too_large(
                path,
                size,
                self.limits.file_size,
            ));
        }
        Ok(Some((input, path)))
    }
}

#[cfg(test)]
mod tests {
    use versionlens_model::DocumentInput;

    use crate::{
        SessionConfig, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryFileSize,
        WorkspaceDiscoveryOptions, enabled_provider_config_from_name, version_lens_session,
    };

    fn options() -> WorkspaceDiscoveryOptions {
        let mut options = WorkspaceDiscoveryOptions::new(vec![]);
        options.overlays.push(
            DocumentInput::new(
                "file:///standalone/package.json",
                "json",
                r#"{"dependencies":{"example":"1.0.0"}}"#,
                None,
            )
            .with_version(7),
        );
        options
    }

    #[test]
    fn standalone_discovery_preserves_the_supplied_snapshot_without_filesystem_roots() {
        let session = version_lens_session(SessionConfig::default());
        let input = options();
        let expected = input.overlays[0].clone();
        let documents = session
            .discover_workspace_documents(input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(documents, [expected]);
    }

    #[test]
    fn standalone_discovery_applies_provider_enablement_and_size_limits() {
        let session = version_lens_session(SessionConfig {
            enabled_providers: vec![
                enabled_provider_config_from_name("cargo").expect("cargo provider"),
            ],
            ..SessionConfig::default()
        });
        assert!(
            session
                .discover_workspace_documents(options())
                .next()
                .is_none()
        );
        let session = version_lens_session(SessionConfig::default());
        let mut excluded = options();
        excluded.exclusions.push("**/package.json".to_owned());
        assert!(
            session
                .discover_workspace_documents(excluded)
                .next()
                .is_some()
        );
        let mut oversized = options();
        oversized.limits.file_size = 1;
        let expected_size = u64::try_from(oversized.overlays[0].text.len()).unwrap();
        let failure = session
            .discover_workspace_documents(oversized)
            .next()
            .unwrap()
            .unwrap_err();
        assert_eq!(
            failure.kind,
            WorkspaceDiscoveryFailureKind::FileTooLarge(Box::new(WorkspaceDiscoveryFileSize {
                size: expected_size,
                limit: 1,
            }))
        );
    }

    #[test]
    fn open_documents_outside_the_workspace_roots_are_still_checked() {
        let root =
            versionlens_test_support::temporary_directory("versionlens-outside-overlay").unwrap();
        let session = version_lens_session(SessionConfig::default());
        let mut input = options();
        input.roots = vec![root.clone()];
        let expected = input.overlays[0].clone();
        let documents = session
            .discover_workspace_documents(input)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(documents, [expected]);
        std::fs::remove_dir_all(root).unwrap();
    }
}
