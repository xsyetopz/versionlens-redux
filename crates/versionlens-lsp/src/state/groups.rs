//! Workspace folders grouped by their settings. Each group has one engine
//! session, so folders with the same settings share resolution and caches,
//! and folders with different settings are resolved apart.

use std::io;

use versionlens_core::{
    SessionConfig, SessionConfigInput, VersionLensSession, version_lens_session,
};

use super::{VersionLensLspState, WorkspaceRoot, configuration::default_workspace_exclusions};

/// The settings of one workspace folder.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FolderSettings {
    pub(crate) config: SessionConfig,
    pub(crate) exclusions: Vec<String>,
}

impl Default for FolderSettings {
    fn default() -> Self {
        Self {
            config: SessionConfigInput::default().into(),
            exclusions: default_workspace_exclusions(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct SessionGroup {
    pub(crate) settings: FolderSettings,
    pub(crate) session: VersionLensSession,
    /// Indexes of the group's folders in the folder scopes.
    scopes: Vec<usize>,
}

impl SessionGroup {
    /// One group with the default settings for every folder scope.
    pub(super) fn initial(scope_count: usize) -> Self {
        let settings = FolderSettings::default();
        Self {
            session: version_lens_session(settings.config.clone()),
            settings,
            scopes: (0..scope_count).collect(),
        }
    }
}

impl VersionLensLspState {
    /// The folders that settings are read for: the workspace folders, or the
    /// root URI of a client that sends no folders.
    pub(super) fn scopes(&self) -> &[WorkspaceRoot] {
        if self.workspace_folders.is_empty() {
            self.root_uri.as_slice()
        } else {
            &self.workspace_folders
        }
    }

    /// The settings of each folder scope, or the one default when there is
    /// no folder.
    fn folder_settings(&self) -> Vec<FolderSettings> {
        if self.scopes().is_empty() {
            vec![self.unscoped_settings.clone()]
        } else {
            self.scopes()
                .iter()
                .map(|root| root.settings.clone())
                .collect()
        }
    }

    /// Sets the settings of each folder scope, in the order of
    /// [`Self::scopes`]. Returns whether any setting changed.
    pub(crate) fn replace_settings(&mut self, settings: Vec<FolderSettings>) -> io::Result<bool> {
        if settings == self.folder_settings() {
            return Ok(false);
        }
        let mut settings = settings.into_iter();
        if self.workspace_folders.is_empty() {
            match self.root_uri.as_mut() {
                Some(root) => root.settings = settings.next().unwrap_or_default(),
                None => self.unscoped_settings = settings.next().unwrap_or_default(),
            }
        } else {
            for root in &mut self.workspace_folders {
                root.settings = settings.next().unwrap_or_default();
            }
        }
        self.regroup()?;
        self.renew_generations()?;
        self.synchronize_workspace_documents();
        Ok(true)
    }

    /// Builds the groups from the folder settings. A group whose settings
    /// did not change keeps its session, so its caches stay warm. A session
    /// is never shared by two groups, because each group's checker sets the
    /// session's open documents to its own.
    pub(super) fn regroup(&mut self) -> io::Result<()> {
        let mut groups: Vec<SessionGroup> = Vec::new();
        for (scope, settings) in self.folder_settings().into_iter().enumerate() {
            if let Some(group) = groups.iter_mut().find(|group| group.settings == settings) {
                group.scopes.push(scope);
                continue;
            }
            let session = match self.groups.iter().find(|group| group.settings == settings) {
                Some(group) => group.session.clone(),
                None => self.new_session(settings.config.clone())?,
            };
            groups.push(SessionGroup {
                settings,
                session,
                scopes: vec![scope],
            });
        }
        for old in &self.groups {
            if !groups.iter().any(|group| group.settings == old.settings) {
                old.session.cancel_pending_resolutions();
            }
        }
        self.groups = groups;
        Ok(())
    }

    fn new_session(&self, config: SessionConfig) -> io::Result<VersionLensSession> {
        let session = version_lens_session(config);
        match &self.modes.cache_directory {
            Some(directory) => session.with_persistent_cache(directory),
            None => Ok(session),
        }
    }

    pub(crate) fn groups(&self) -> &[SessionGroup] {
        &self.groups
    }

    /// The group of the folder that contains a document. A document outside
    /// every folder belongs to the first group.
    pub(crate) fn group_of(&self, document_uri: &str) -> usize {
        let Ok(document_uri) = document_uri.parse() else {
            return 0;
        };
        self.scopes()
            .iter()
            .enumerate()
            .filter(|(_, root)| root.contains(&document_uri))
            .max_by_key(|(_, root)| root.uri.path().as_str().len())
            .and_then(|(scope, _)| {
                self.groups
                    .iter()
                    .position(|group| group.scopes.contains(&scope))
            })
            .unwrap_or(0)
    }

    pub(crate) fn session_for(&self, document_uri: &str) -> &VersionLensSession {
        &self.groups[self.group_of(document_uri)].session
    }

    /// The folder roots of a group.
    pub(super) fn group_roots(&self, group: usize) -> Vec<&WorkspaceRoot> {
        let scopes = self.scopes();
        self.groups[group]
            .scopes
            .iter()
            .filter_map(|scope| scopes.get(*scope))
            .collect()
    }

    /// Stores session results in `directory` as well as in memory. When any
    /// session cannot open it, nothing changes and sessions stay in memory.
    pub(crate) fn use_persistent_cache(&mut self, directory: &std::path::Path) -> io::Result<()> {
        let sessions = self
            .groups
            .iter()
            .map(|group| group.session.clone().with_persistent_cache(directory))
            .collect::<io::Result<Vec<_>>>()?;
        for (group, session) in self.groups.iter_mut().zip(sessions) {
            group.session = session;
        }
        self.modes.cache_directory = Some(directory.to_owned());
        Ok(())
    }

    pub(crate) fn cancel_pending_resolutions(&self) {
        for group in &self.groups {
            group.session.cancel_pending_resolutions();
        }
    }
}
