use std::collections::{HashSet, VecDeque};
use std::fs::{self, DirEntry, Metadata, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

use ignore::{IncrementalIgnore, WalkBuilder};
use versionlens_model::{DocumentInput, Ecosystem, ManifestKind, ecosystem_for_manifest};

use super::VersionLensSession;

mod failure;
mod options;
mod overlays;
mod support;

pub use options::{
    WorkspaceDiscoveryCancellation, WorkspaceDiscoveryLimits, WorkspaceDiscoveryOptions,
    WorkspaceProviderExclusion,
};

use support::ReadFailure;
use support::{exclusion_matches, language_id, normalize_absolute, read_bounded_utf8, slash_path};
pub use support::{
    exclusion_matches as workspace_exclusion_matches, slash_path as workspace_exclusion_path,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceDiscoveryIoOperation {
    ResolvePath,
    ReadDirectory,
    ReadMetadata,
    ReadFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDiscoveryFileSize {
    pub size: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceDiscoveryFailureKind {
    Io {
        operation: WorkspaceDiscoveryIoOperation,
        error_kind: io::ErrorKind,
    },
    InvalidOverlayUri,
    InvalidFileUri,
    OutsideWorkspace,
    EscapesWorkspace,
    InvalidUtf8,
    FileTooLarge(Box<WorkspaceDiscoveryFileSize>),
    DepthLimitExceeded {
        limit: usize,
    },
    FileLimitExceeded {
        limit: usize,
    },
    EntryLimitExceeded {
        limit: usize,
    },
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceDiscoveryFailure {
    pub path: Option<PathBuf>,
    pub kind: WorkspaceDiscoveryFailureKind,
}

impl WorkspaceDiscoveryFailure {
    pub(super) fn io(
        path: PathBuf,
        operation: WorkspaceDiscoveryIoOperation,
        error: &io::Error,
    ) -> Self {
        Self {
            path: Some(path),
            kind: WorkspaceDiscoveryFailureKind::Io {
                operation,
                error_kind: error.kind(),
            },
        }
    }

    pub(super) fn at(path: PathBuf, kind: WorkspaceDiscoveryFailureKind) -> Self {
        Self {
            path: Some(path),
            kind,
        }
    }

    pub(super) fn file_too_large(path: PathBuf, size: u64, limit: u64) -> Self {
        Self::at(
            path,
            WorkspaceDiscoveryFailureKind::FileTooLarge(Box::new(WorkspaceDiscoveryFileSize {
                size,
                limit,
            })),
        )
    }
}

#[derive(Debug)]
struct DiscoveryRoot {
    declared_path: PathBuf,
    path: PathBuf,
    uri: String,
}

#[derive(Debug)]
struct DirectoryFrame {
    root_index: usize,
    depth: usize,
    path: PathBuf,
    entries: ReadDir,
}

struct ResolvedDiskEntry {
    root_index: usize,
    depth: usize,
    path: PathBuf,
    canonical: PathBuf,
    metadata: Metadata,
    symlink: bool,
}

type DiscoveryResult = Result<DocumentInput, WorkspaceDiscoveryFailure>;
type DiscoveryStep = Option<DiscoveryResult>;

struct DiscoveryTraversal {
    overlays: VecDeque<DocumentInput>,
    pending: VecDeque<DiscoveryResult>,
    directories: Vec<DirectoryFrame>,
    seen_paths: HashSet<PathBuf>,
    seen_directories: HashSet<PathBuf>,
    next_root: usize,
    yielded_files: usize,
    visited_entries: usize,
    overlays_complete: bool,
    stopped: bool,
}

pub struct WorkspaceDiscovery {
    session: VersionLensSession,
    roots: Vec<DiscoveryRoot>,
    exclusions: Vec<String>,
    provider_exclusions: Vec<WorkspaceProviderExclusion>,
    limits: WorkspaceDiscoveryLimits,
    cancellation: WorkspaceDiscoveryCancellation,
    git_ignores: Vec<IncrementalIgnore>,
    traversal: DiscoveryTraversal,
    overlay_only: bool,
}

impl VersionLensSession {
    pub fn discover_workspace_documents(
        &self,
        options: WorkspaceDiscoveryOptions,
    ) -> WorkspaceDiscovery {
        WorkspaceDiscovery::new(self, options)
    }
}

impl WorkspaceDiscovery {
    pub(super) fn skip_paths(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if let Some((_, path)) = self.root_for_path(path) {
                self.traversal.seen_paths.insert(path);
            }
        }
    }

    fn new(session: &VersionLensSession, options: WorkspaceDiscoveryOptions) -> Self {
        let overlay_only = options.roots.is_empty();
        let mut roots = Vec::new();
        let mut pending = VecDeque::new();
        let mut seen_roots = HashSet::new();
        for requested in options.roots {
            match fs::canonicalize(&requested) {
                Ok(path) => {
                    if !seen_roots.insert(path.clone()) {
                        continue;
                    }
                    match crate::workspace_file_uri(&path) {
                        Some(uri) => roots.push(DiscoveryRoot {
                            declared_path: normalize_absolute(&requested)
                                .unwrap_or_else(|| path.clone()),
                            path,
                            uri,
                        }),
                        None => pending.push_back(Err(WorkspaceDiscoveryFailure::at(
                            path,
                            WorkspaceDiscoveryFailureKind::InvalidFileUri,
                        ))),
                    }
                }
                Err(error) => pending.push_back(Err(WorkspaceDiscoveryFailure::io(
                    requested,
                    WorkspaceDiscoveryIoOperation::ResolvePath,
                    &error,
                ))),
            }
        }

        let canonical_roots = roots.iter().map(|root| &root.path).collect::<Vec<_>>();
        let mut ignore_builder = WalkBuilder::from_iter(canonical_roots);
        ignore_builder
            .hidden(false)
            .parents(false)
            .ignore(false)
            .git_ignore(true)
            .git_global(false)
            .git_exclude(true)
            .require_git(true)
            .follow_links(false);
        let git_ignores = ignore_builder.build_matchers();

        Self {
            session: session.clone(),
            roots,
            exclusions: options.exclusions,
            provider_exclusions: options.provider_exclusions,
            limits: options.limits,
            cancellation: options.cancellation,
            git_ignores,
            traversal: DiscoveryTraversal {
                overlays: options.overlays.into(),
                pending,
                directories: Vec::new(),
                seen_paths: HashSet::new(),
                seen_directories: HashSet::new(),
                next_root: 0,
                yielded_files: 0,
                visited_entries: 0,
                overlays_complete: false,
                stopped: false,
            },
            overlay_only,
        }
    }

    fn cancelled(&mut self) -> DiscoveryStep {
        if !self.traversal.stopped && self.cancellation.is_cancelled() {
            self.traversal.stopped = true;
            return Some(Err(WorkspaceDiscoveryFailure {
                path: None,
                kind: WorkspaceDiscoveryFailureKind::Cancelled,
            }));
        }
        None
    }

    fn root_for_path(&self, path: &Path) -> Option<(usize, PathBuf)> {
        let normalized = normalize_absolute(path)?;
        self.roots
            .iter()
            .enumerate()
            .filter_map(|(index, root)| {
                normalized
                    .strip_prefix(&root.path)
                    .or_else(|_| normalized.strip_prefix(&root.declared_path))
                    .ok()
                    .map(|relative| {
                        (
                            index,
                            root.path.join(relative),
                            root.declared_path.components().count(),
                        )
                    })
            })
            .max_by_key(|(_, _, root_depth)| *root_depth)
            .map(|(index, canonical_path, _)| (index, canonical_path))
    }

    fn supported(&self, input: &DocumentInput, apply_provider_exclusions: bool) -> bool {
        let kind = self.session.classify_document(input);
        if kind == ManifestKind::Unknown {
            return false;
        }
        ecosystem_for_manifest(kind).is_some_and(|ecosystem| {
            self.session.provider_enabled_for_manifest(kind, ecosystem)
                && (!apply_provider_exclusions || !self.provider_excluded(input, ecosystem))
        })
    }

    fn provider_excluded(&self, input: &DocumentInput, ecosystem: Ecosystem) -> bool {
        let Some(path) = crate::workspace_path(&input.uri) else {
            return false;
        };
        let root = input
            .workspace_root
            .as_deref()
            .and_then(crate::workspace_path);
        let relative = root.as_ref().and_then(|root| path.strip_prefix(root).ok());
        let candidate = slash_path(relative.unwrap_or(&path));
        self.provider_exclusions.iter().any(|exclusion| {
            exclusion.ecosystem == ecosystem
                && exclusion
                    .patterns
                    .iter()
                    .any(|pattern| exclusion_matches(pattern, &candidate, false))
        })
    }

    fn accept(
        &mut self,
        input: DocumentInput,
        path: PathBuf,
    ) -> Result<DocumentInput, WorkspaceDiscoveryFailure> {
        if self.traversal.yielded_files >= self.limits.max_files {
            self.traversal.stopped = true;
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::FileLimitExceeded {
                    limit: self.limits.max_files,
                },
            ));
        }
        self.traversal.yielded_files += 1;
        Ok(input)
    }

    fn visit_entry(&mut self, path: PathBuf) -> Result<(), WorkspaceDiscoveryFailure> {
        if self.traversal.visited_entries >= self.limits.max_visited_entries {
            self.traversal.stopped = true;
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::EntryLimitExceeded {
                    limit: self.limits.max_visited_entries,
                },
            ));
        }
        self.traversal.visited_entries += 1;
        Ok(())
    }

    fn excluded(&mut self, root_index: usize, path: &Path, directory: bool) -> bool {
        let root = &self.roots[root_index];
        let Ok(relative) = path.strip_prefix(&root.path) else {
            return true;
        };
        if self.git_ignores[root_index]
            .matched(relative, directory)
            .is_ignore()
        {
            return true;
        }
        let relative = slash_path(relative);
        self.exclusions
            .iter()
            .any(|pattern| exclusion_matches(pattern, &relative, directory))
    }

    fn open_next_root(&mut self) -> Option<WorkspaceDiscoveryFailure> {
        while self.traversal.next_root < self.roots.len() {
            let root_index = self.traversal.next_root;
            self.traversal.next_root += 1;
            let root = &self.roots[root_index];
            if !self.traversal.seen_directories.insert(root.path.clone()) {
                continue;
            }
            match fs::read_dir(&root.path) {
                Ok(entries) => {
                    self.traversal.directories.push(DirectoryFrame {
                        root_index,
                        depth: 0,
                        path: root.path.clone(),
                        entries,
                    });
                    return None;
                }
                Err(error) => {
                    return Some(WorkspaceDiscoveryFailure::io(
                        root.path.clone(),
                        WorkspaceDiscoveryIoOperation::ReadDirectory,
                        &error,
                    ));
                }
            }
        }
        None
    }

    fn next_disk(&mut self) -> DiscoveryStep {
        loop {
            if let Some(cancelled) = self.cancelled() {
                return Some(cancelled);
            }
            let raw = match self.next_disk_entry()? {
                Ok(raw) => raw,
                Err(failure) => return Some(Err(failure)),
            };
            let entry = match self.resolve_disk_entry(raw) {
                Ok(entry) => entry,
                Err(failure) => return Some(Err(failure)),
            };
            let result = if entry.metadata.is_dir() {
                self.visit_directory(entry)
            } else {
                self.visit_file(entry)
            };
            if result.is_some() {
                return result;
            }
        }
    }

    fn next_disk_entry(
        &mut self,
    ) -> Option<Result<(usize, usize, DirEntry), WorkspaceDiscoveryFailure>> {
        loop {
            if self.traversal.directories.is_empty() {
                if let Some(failure) = self.open_next_root() {
                    return Some(Err(failure));
                }
                if self.traversal.directories.is_empty() {
                    return None;
                }
            }
            let frame = self.traversal.directories.last_mut()?;
            match frame.entries.next() {
                Some(Ok(entry)) => return Some(Ok((frame.root_index, frame.depth, entry))),
                Some(Err(error)) => {
                    return Some(Err(WorkspaceDiscoveryFailure::io(
                        frame.path.clone(),
                        WorkspaceDiscoveryIoOperation::ReadDirectory,
                        &error,
                    )));
                }
                None => {
                    self.traversal.directories.pop();
                }
            }
        }
    }

    fn resolve_disk_entry(
        &mut self,
        (root_index, depth, entry): (usize, usize, DirEntry),
    ) -> Result<ResolvedDiskEntry, WorkspaceDiscoveryFailure> {
        let path = entry.path();
        self.visit_entry(path.clone())?;
        let symlink_metadata = fs::symlink_metadata(&path).map_err(|error| {
            WorkspaceDiscoveryFailure::io(
                path.clone(),
                WorkspaceDiscoveryIoOperation::ReadMetadata,
                &error,
            )
        })?;
        let canonical = fs::canonicalize(&path).map_err(|error| {
            WorkspaceDiscoveryFailure::io(
                path.clone(),
                WorkspaceDiscoveryIoOperation::ResolvePath,
                &error,
            )
        })?;
        if !canonical.starts_with(&self.roots[root_index].path) {
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::EscapesWorkspace,
            ));
        }
        let symlink = symlink_metadata.file_type().is_symlink();
        let metadata = if symlink {
            fs::metadata(&canonical).map_err(|error| {
                WorkspaceDiscoveryFailure::io(
                    path.clone(),
                    WorkspaceDiscoveryIoOperation::ReadMetadata,
                    &error,
                )
            })?
        } else {
            symlink_metadata
        };
        Ok(ResolvedDiskEntry {
            root_index,
            depth,
            path,
            canonical,
            metadata,
            symlink,
        })
    }

    fn visit_directory(&mut self, entry: ResolvedDiskEntry) -> DiscoveryStep {
        if self.excluded(entry.root_index, &entry.path, true) || entry.symlink {
            return None;
        }
        if entry.depth >= self.limits.max_depth {
            return Some(Err(WorkspaceDiscoveryFailure::at(
                entry.path,
                WorkspaceDiscoveryFailureKind::DepthLimitExceeded {
                    limit: self.limits.max_depth,
                },
            )));
        }
        if !self
            .traversal
            .seen_directories
            .insert(entry.canonical.clone())
        {
            return None;
        }
        match fs::read_dir(&entry.canonical) {
            Ok(entries) => {
                self.traversal.directories.push(DirectoryFrame {
                    root_index: entry.root_index,
                    depth: entry.depth + 1,
                    path: entry.path,
                    entries,
                });
                None
            }
            Err(error) => Some(Err(WorkspaceDiscoveryFailure::io(
                entry.path,
                WorkspaceDiscoveryIoOperation::ReadDirectory,
                &error,
            ))),
        }
    }

    fn visit_file(&mut self, entry: ResolvedDiskEntry) -> DiscoveryStep {
        if !entry.metadata.is_file()
            || self.excluded(entry.root_index, &entry.path, false)
            || !self.traversal.seen_paths.insert(entry.path.clone())
        {
            return None;
        }
        let Some(uri) = crate::workspace_file_uri(&entry.path) else {
            return Some(Err(WorkspaceDiscoveryFailure::at(
                entry.path,
                WorkspaceDiscoveryFailureKind::InvalidFileUri,
            )));
        };
        let root_uri = self.roots[entry.root_index].uri.clone();
        let probe = DocumentInput::new(
            uri.clone(),
            language_id(&entry.path),
            String::new(),
            Some(root_uri.clone()),
        );
        if !self.supported(&probe, true) {
            return None;
        }
        if entry.metadata.len() > self.limits.max_file_size {
            return Some(Err(WorkspaceDiscoveryFailure::file_too_large(
                entry.path,
                entry.metadata.len(),
                self.limits.max_file_size,
            )));
        }
        let text = match read_bounded_utf8(&entry.path, self.limits.max_file_size) {
            Ok(text) => text,
            Err(ReadFailure::Io(error)) => {
                return Some(Err(WorkspaceDiscoveryFailure::io(
                    entry.path,
                    WorkspaceDiscoveryIoOperation::ReadFile,
                    &error,
                )));
            }
            Err(ReadFailure::TooLarge(size)) => {
                return Some(Err(WorkspaceDiscoveryFailure::file_too_large(
                    entry.path,
                    size,
                    self.limits.max_file_size,
                )));
            }
            Err(ReadFailure::InvalidUtf8) => {
                return Some(Err(WorkspaceDiscoveryFailure::at(
                    entry.path,
                    WorkspaceDiscoveryFailureKind::InvalidUtf8,
                )));
            }
        };
        Some(self.accept(
            DocumentInput::new(uri, language_id(&entry.path), text, Some(root_uri)),
            entry.path,
        ))
    }
}

impl Iterator for WorkspaceDiscovery {
    type Item = DiscoveryResult;

    fn next(&mut self) -> Option<Self::Item> {
        if self.traversal.stopped {
            return None;
        }
        if let Some(cancelled) = self.cancelled() {
            return Some(cancelled);
        }
        if let Some(item) = self.traversal.pending.pop_front() {
            return Some(item);
        }
        if !self.traversal.overlays_complete {
            if let Some(item) = self.next_overlay() {
                return Some(item);
            }
        }
        self.next_disk()
    }
}

#[cfg(test)]
mod tests;
