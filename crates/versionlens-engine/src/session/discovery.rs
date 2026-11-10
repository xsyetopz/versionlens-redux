use std::collections::{HashSet, VecDeque};
use std::fs::{self, DirEntry, Metadata, ReadDir};
use std::io;
use std::path::{Path, PathBuf};

use ignore::{IncrementalIgnore, WalkBuilder};
use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};
use versionlens_parsers::ecosystem_for_manifest;

use super::VersionLensSession;

mod failure;
mod options;
mod overlays;
mod support;

pub use options::WorkspaceDiscoveryOptions;
pub(crate) use options::{
    WorkspaceDiscoveryCancellation, WorkspaceDiscoveryLimits, WorkspaceProviderExclusion,
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
/// Why discovery failed on a path.
pub enum WorkspaceDiscoveryFailureKind {
    /// An I/O call failed.
    Io {
        /// The call that failed.
        operation: WorkspaceDiscoveryIoOperation,
        /// The kind of the error.
        error_kind: io::ErrorKind,
    },
    /// An overlay URI is not a file URI.
    InvalidOverlayUri,
    /// A file URI is invalid.
    InvalidFileUri,
    /// The path leaves the workspace.
    EscapesWorkspace,
    /// The file is not valid UTF-8.
    InvalidUtf8,
    /// The file is larger than the limit.
    FileTooLarge(Box<WorkspaceDiscoveryFileSize>),
    /// The directories are nested deeper than the limit.
    DepthLimitExceeded {
        /// The depth limit.
        limit: usize,
    },
    /// More files than the limit.
    FileLimitExceeded {
        /// The file limit.
        limit: usize,
    },
    /// More directory entries than the limit.
    EntryLimitExceeded {
        /// The entry limit.
        limit: usize,
    },
    /// Discovery was cancelled.
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A path that discovery could not read.
pub struct WorkspaceDiscoveryFailure {
    /// The path, when the failure has one.
    pub path: Option<PathBuf>,
    /// The reason.
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

impl std::fmt::Debug for WorkspaceDiscovery {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorkspaceDiscovery")
            .field("roots", &self.roots)
            .finish_non_exhaustive()
    }
}

impl VersionLensSession {
    /// Returns an iterator over the documents that `options` selects.
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

    /// Checks one changed path as a full scan would: the path's document URI,
    /// and the document, a failure, or `None` when a scan skips the path.
    /// Returns `None` for a path outside every root.
    pub(super) fn rescan_path(&mut self, path: &Path) -> Option<(String, Option<DiscoveryResult>)> {
        let (root_index, path) = self.root_for_path(path)?;
        let uri = crate::workspace_file_uri(&path)?;
        if self.ancestor_excluded(root_index, &path) {
            return Some((uri, None));
        }
        let entry = fs::canonicalize(&path)
            .ok()
            .filter(|canonical| canonical.starts_with(&self.roots[root_index].path))
            .and_then(|canonical| Some((fs::metadata(&canonical).ok()?, canonical)));
        let Some((metadata, canonical)) = entry else {
            return Some((uri, None));
        };
        self.traversal.seen_paths.remove(&path);
        self.traversal.yielded_files = 0;
        self.traversal.stopped = false;
        let result = self.visit_file(ResolvedDiskEntry {
            root_index,
            depth: 0,
            path,
            canonical,
            metadata,
            symlink: false,
        });
        Some((uri, result))
    }

    /// Whether a scan skips `path`, a directory, because it lies outside every
    /// root or is ignored or excluded.
    pub fn skips_directory(&mut self, path: &Path) -> bool {
        self.root_for_path(path).is_none_or(|(root_index, path)| {
            path != self.roots[root_index].path && self.excluded(root_index, &path, true)
        })
    }

    /// Whether a scan skips `path`, a file, because it is ignored or excluded
    /// under its root. A file outside every root is not skipped.
    pub fn skips_file(&mut self, path: &Path) -> bool {
        self.root_for_path(path).is_some_and(|(root_index, path)| {
            self.excluded(root_index, &path, false) || self.ancestor_excluded(root_index, &path)
        })
    }

    /// Whether a directory between `path` and its root is ignored or excluded.
    /// A scan never enters such a directory, but a glob that names only the
    /// directory, such as `**/.claude`, does not match the files inside it.
    fn ancestor_excluded(&mut self, root_index: usize, path: &Path) -> bool {
        let root = self.roots[root_index].path.clone();
        path.ancestors()
            .skip(1)
            .take_while(|ancestor| *ancestor != root)
            .any(|ancestor| self.excluded(root_index, ancestor, true))
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
            .ignore(true)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true)
            .require_git(false)
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
        if self.traversal.yielded_files >= self.limits.files {
            self.traversal.stopped = true;
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::FileLimitExceeded {
                    limit: self.limits.files,
                },
            ));
        }
        self.traversal.yielded_files += 1;
        Ok(input)
    }

    fn visit_entry(&mut self, path: PathBuf) -> Result<(), WorkspaceDiscoveryFailure> {
        if self.traversal.visited_entries >= self.limits.visited_entries {
            self.traversal.stopped = true;
            return Err(WorkspaceDiscoveryFailure::at(
                path,
                WorkspaceDiscoveryFailureKind::EntryLimitExceeded {
                    limit: self.limits.visited_entries,
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
        if entry.depth >= self.limits.depth {
            return Some(Err(WorkspaceDiscoveryFailure::at(
                entry.path,
                WorkspaceDiscoveryFailureKind::DepthLimitExceeded {
                    limit: self.limits.depth,
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
        if entry.metadata.len() > self.limits.file_size {
            return Some(Err(WorkspaceDiscoveryFailure::file_too_large(
                entry.path,
                entry.metadata.len(),
                self.limits.file_size,
            )));
        }
        let text = match read_bounded_utf8(&entry.path, self.limits.file_size) {
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
                    self.limits.file_size,
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
        if !self.traversal.overlays_complete
            && let Some(item) = self.next_overlay()
        {
            return Some(item);
        }
        self.next_disk()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{self, create_dir_all, write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

    use super::{
        WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryFileSize, WorkspaceDiscoveryIoOperation,
        WorkspaceDiscoveryLimits, WorkspaceDiscoveryOptions,
    };
    use crate::{EnabledProviderConfig, FilePatternConfig, ProviderSettings, VersionLensSession};

    static NEXT_WORKSPACE: AtomicU64 = AtomicU64::new(0);

    struct TestWorkspace {
        root: PathBuf,
    }

    impl TestWorkspace {
        fn new(name: &str) -> Self {
            let id = NEXT_WORKSPACE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "versionlens-discovery-{name}-{}-{id}",
                std::process::id()
            ));
            create_dir_all(&root).unwrap();
            Self { root }
        }

        fn write(&self, relative: &str, text: impl AsRef<[u8]>) -> PathBuf {
            let path = self.root.join(relative);
            create_dir_all(path.parent().unwrap()).unwrap();
            write(&path, text).unwrap();
            path
        }

        fn uri(&self, relative: &str) -> String {
            crate::workspace_file_uri(&self.root.canonicalize().unwrap().join(relative)).unwrap()
        }
    }

    impl Drop for TestWorkspace {
        fn drop(&mut self) {
            drop(fs::remove_dir_all(&self.root));
        }
    }

    fn session() -> VersionLensSession {
        crate::test_support::test_session(false)
    }

    fn options(workspace: &TestWorkspace) -> WorkspaceDiscoveryOptions {
        WorkspaceDiscoveryOptions::new(vec![workspace.root.clone()])
    }

    fn successful_uris(
        results: &[Result<DocumentInput, super::WorkspaceDiscoveryFailure>],
    ) -> Vec<String> {
        let mut uris = results
            .iter()
            .filter_map(|result| result.as_ref().ok().map(|input| input.uri.clone()))
            .collect::<Vec<_>>();
        uris.sort();
        uris
    }

    #[test]
    fn discovers_unopened_nested_and_runtime_manifests_incrementally() {
        let workspace = TestWorkspace::new("nested");
        workspace.write("packages/a/package.json", r#"{"dependencies":{}}"#);
        workspace.write("tools/.nvmrc", "22\n");
        workspace.write("notes/readme.txt", "not a manifest");

        let results = session()
            .discover_workspace_documents(options(&workspace))
            .collect::<Vec<_>>();

        assert!(results.iter().all(Result::is_ok));
        assert_eq!(
            successful_uris(&results),
            [
                workspace.uri("packages/a/package.json"),
                workspace.uri("tools/.nvmrc")
            ]
        );
    }

    #[test]
    fn exclusions_prune_disk_but_keep_explicit_overlays() {
        let workspace = TestWorkspace::new("exclusions");
        workspace.write("apps/included/package.json", "{}");
        workspace.write("apps/excluded/package.json", "{}");
        let mut request = options(&workspace);
        request.exclusions = vec!["**/excluded/**".to_owned()];
        request.overlays.push(DocumentInput::new(
            workspace.uri("apps/excluded/new/package.json"),
            "json",
            r#"{"dependencies":{"dirty":"1"}}"#,
            None,
        ));

        let results = session()
            .discover_workspace_documents(request)
            .collect::<Vec<_>>();

        assert_eq!(
            successful_uris(&results),
            [
                workspace.uri("apps/excluded/new/package.json"),
                workspace.uri("apps/included/package.json")
            ]
        );
    }

    #[test]
    fn disabled_provider_is_rejected_before_the_file_size_or_read_gate() {
        let workspace = TestWorkspace::new("disabled");
        workspace.write("Cargo.toml", vec![b'x'; 512]);
        workspace.write("package.json", "{}");
        let mut config = crate::test_support::session_config(ProviderSettings::default(), false);
        config.enabled_providers = vec![EnabledProviderConfig {
            ecosystem: Ecosystem::Npm,
            manifest_kind: Some(ManifestKind::NpmPackageJson),
        }];
        let session = VersionLensSession::new(config);
        let mut request = options(&workspace);
        request.limits.file_size = 8;

        let results = session
            .discover_workspace_documents(request)
            .collect::<Vec<_>>();

        assert!(results.iter().all(Result::is_ok));
        assert_eq!(successful_uris(&results), [workspace.uri("package.json")]);
    }

    #[test]
    fn provider_exclusions_apply_to_disk_without_hiding_supported_overlays() {
        let workspace = TestWorkspace::new("provider-exclusions");
        workspace.write("obj/generated.csproj", "<Project />");
        workspace.write("obj/package.json", "{}");
        workspace.write("src/application.csproj", "<Project />");
        let mut request = options(&workspace);
        request
            .provider_exclusions
            .push(super::WorkspaceProviderExclusion {
                ecosystem: Ecosystem::Dotnet,
                patterns: vec!["**/obj/**".to_owned()],
            });
        request.overlays = vec![
            DocumentInput::new(
                workspace.uri("obj/dirty.csproj"),
                "xml",
                "<Project />",
                None,
            ),
            DocumentInput::new(workspace.uri("obj/new/package.json"), "json", "{}", None),
        ];
        let results = session()
            .discover_workspace_documents(request)
            .collect::<Vec<_>>();
        assert!(results.iter().all(Result::is_ok));
        assert_eq!(
            successful_uris(&results),
            [
                workspace.uri("obj/new/package.json"),
                workspace.uri("obj/package.json"),
                workspace.uri("src/application.csproj"),
            ]
        );
    }

    #[test]
    fn git_ignores_nested_rules_and_repository_excludes_but_keep_hidden_manifests() {
        let workspace = TestWorkspace::new("git-ignore");
        workspace.write(".git/HEAD", "ref: refs/heads/main\n");
        workspace.write(".git/info/exclude", "info/package.json\n");
        workspace.write(".gitignore", "ignored/\n");
        workspace.write("nested/.gitignore", "*.json\n!package.json\n");
        workspace.write("ignored/package.json", "{}");
        workspace.write("info/package.json", "{}");
        workspace.write("nested/package.json", "{}");
        workspace.write("nested/other.json", "{}");
        workspace.write("tools/.nvmrc", "22\n");

        let results = session()
            .discover_workspace_documents(options(&workspace))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        let mut uris = results
            .iter()
            .map(|input| input.uri.clone())
            .collect::<Vec<_>>();
        uris.sort();
        assert_eq!(
            uris,
            [
                workspace.uri("nested/package.json"),
                workspace.uri("tools/.nvmrc")
            ]
        );
    }

    #[test]
    fn ignore_files_apply_without_a_git_repository() {
        let workspace = TestWorkspace::new("ignore-files");
        workspace.write(".gitignore", "worktrees/\n");
        workspace.write(".ignore", "scratch/\n");
        workspace.write("worktrees/copy/package.json", "{}");
        workspace.write("scratch/package.json", "{}");
        workspace.write("app/package.json", "{}");

        let mut discovery = session().discover_workspace_documents(options(&workspace));
        let root = workspace.root.canonicalize().unwrap();
        assert!(discovery.skips_directory(&root.join("worktrees/copy")));
        assert!(discovery.skips_directory(&root.join("scratch")));
        assert!(!discovery.skips_directory(&root.join("app")));
        assert!(!discovery.skips_directory(&root));
        assert!(discovery.skips_file(&root.join("worktrees/copy/package.json")));
        assert!(discovery.skips_file(&root.join("scratch/package.json")));
        assert!(!discovery.skips_file(&root.join("app/package.json")));
        assert!(!discovery.skips_file(Path::new("/outside/worktrees/package.json")));
        let results = discovery.collect::<Vec<_>>();

        assert_eq!(
            successful_uris(&results),
            [workspace.uri("app/package.json")]
        );
    }

    #[test]
    fn rescanned_paths_follow_the_scan_rules() {
        let workspace = TestWorkspace::new("rescan");
        workspace.write(".gitignore", "ignored/\n");
        let kept = workspace.write("app/package.json", "{}");
        let ignored = workspace.write("ignored/package.json", "{}");
        let other = workspace.write("app/readme.txt", "");
        let mut discovery = session().discover_workspace_documents(options(&workspace));

        let (uri, result) = discovery.rescan_path(&kept).unwrap();
        assert_eq!(uri, workspace.uri("app/package.json"));
        assert_eq!(result.unwrap().unwrap().uri, uri);
        assert!(discovery.rescan_path(&kept).unwrap().1.is_some());
        assert!(discovery.rescan_path(&ignored).unwrap().1.is_none());
        assert!(discovery.rescan_path(&other).unwrap().1.is_none());
        fs::remove_file(&kept).unwrap();
        assert_eq!(
            discovery
                .rescan_path(&kept)
                .map(|(uri, result)| (uri, result.is_none())),
            Some((workspace.uri("app/package.json"), true))
        );
        assert!(
            discovery
                .rescan_path(Path::new("/elsewhere/package.json"))
                .is_none()
        );
    }

    #[test]
    fn a_glob_for_a_directory_skips_the_files_inside_it() {
        let workspace = TestWorkspace::new("directory-glob");
        let inside = workspace.write(".claude/worktrees/copy/package.json", "{}");
        let kept = workspace.write("app/package.json", "{}");
        let mut options = options(&workspace);
        options.exclusions = vec!["**/.claude".to_owned()];
        let mut discovery = session().discover_workspace_documents(options);

        assert!(discovery.skips_file(&inside));
        assert!(!discovery.skips_file(&kept));
        assert!(discovery.rescan_path(&inside).unwrap().1.is_none());
        assert!(discovery.rescan_path(&kept).unwrap().1.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn internal_directory_symlinks_are_never_followed() {
        use std::os::unix::fs::symlink;

        let workspace = TestWorkspace::new("internal-symlink");
        workspace.write("real/package.json", "{}");
        symlink(workspace.root.join("real"), workspace.root.join("linked")).unwrap();

        let results = session()
            .discover_workspace_documents(options(&workspace))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(
            results
                .iter()
                .map(|input| input.uri.as_str())
                .collect::<Vec<_>>(),
            [workspace.uri("real/package.json")]
        );
    }

    #[test]
    fn dirty_and_new_overlays_replace_disk_and_preserve_versions() {
        let workspace = TestWorkspace::new("overlays");
        workspace.write("package.json", r#"{"version":"disk"}"#);
        let mut request = options(&workspace);
        request.overlays = vec![
            DocumentInput::new(
                workspace.uri("package.json"),
                "json",
                r#"{"version":"dirty"}"#,
                None,
            )
            .with_version(7),
            DocumentInput::new(
                workspace.uri("packages/new/package.json"),
                "json",
                r#"{"version":"new"}"#,
                None,
            )
            .with_version(1),
        ];

        let results = session()
            .discover_workspace_documents(request)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].text, r#"{"version":"dirty"}"#);
        assert_eq!(results[0].version, Some(7));
        assert_eq!(results[1].text, r#"{"version":"new"}"#);
        assert_eq!(results[1].version, Some(1));
        assert!(results.iter().all(|input| input.workspace_root.is_some()));
    }

    #[test]
    fn configured_patterns_use_escaped_workspace_file_uris() {
        let workspace = TestWorkspace::new("escaped uri");
        let path = workspace.write("ci files/dépendances #1.custom", "{}");
        let config = crate::test_support::session_config(
            ProviderSettings {
                file_patterns: vec![FilePatternConfig {
                    manifest_kind: ManifestKind::NpmPackageJson,
                    pattern: "ci files/dépendances #?.custom".to_owned(),
                }],
                ..Default::default()
            },
            false,
        );

        let input = VersionLensSession::new(config)
            .discover_workspace_documents(options(&workspace))
            .next()
            .unwrap()
            .unwrap();

        assert_eq!(
            input.uri,
            crate::workspace_file_uri(&path.canonicalize().unwrap()).unwrap()
        );
        assert!(input.uri.contains("%20"));
        assert!(input.uri.contains("%C3%A9"));
        assert!(input.uri.contains("%23"));
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_cannot_escape_the_workspace() {
        use std::os::unix::fs::symlink;

        let workspace = TestWorkspace::new("symlink-root");
        let outside = TestWorkspace::new("symlink-outside");
        outside.write("package.json", "{}");
        symlink(&outside.root, workspace.root.join("linked")).unwrap();
        workspace.write("package.json", "{}");

        let results = session()
            .discover_workspace_documents(options(&workspace))
            .collect::<Vec<_>>();

        assert_eq!(successful_uris(&results), [workspace.uri("package.json")]);
        assert!(results.iter().any(|result| matches!(
            result,
            Err(failure) if failure.kind == WorkspaceDiscoveryFailureKind::EscapesWorkspace
        )));
    }

    #[test]
    fn invalid_utf8_and_oversize_failures_do_not_hide_later_documents() {
        let workspace = TestWorkspace::new("recover");
        workspace.write("bad/package.json", [0xff, 0xfe]);
        workspace.write("large/Cargo.toml", vec![b'x'; 64]);
        workspace.write("good/.nvmrc", "22");
        let mut request = options(&workspace);
        request.limits.file_size = 16;

        let results = session()
            .discover_workspace_documents(request)
            .collect::<Vec<_>>();

        assert_eq!(successful_uris(&results), [workspace.uri("good/.nvmrc")]);
        assert!(results.iter().any(|result| matches!(
            result,
            Err(failure) if failure.kind == WorkspaceDiscoveryFailureKind::InvalidUtf8
        )));
        assert!(results.iter().any(|result| matches!(
            result,
            Err(failure)
                if failure.kind == WorkspaceDiscoveryFailureKind::FileTooLarge(Box::new(
                    WorkspaceDiscoveryFileSize { size: 64, limit: 16 }
                ))
        )));
    }

    #[test]
    fn depth_and_file_count_bounds_are_explicit() {
        let workspace = TestWorkspace::new("bounds");
        workspace.write("one/package.json", "{}");
        workspace.write("two/package.json", "{}");
        workspace.write("deep/nested/package.json", "{}");
        let mut depth_request = options(&workspace);
        depth_request.limits = WorkspaceDiscoveryLimits {
            files: 10,
            visited_entries: 100,
            depth: 1,
            file_size: 1024,
        };

        let depth_results = session()
            .discover_workspace_documents(depth_request)
            .collect::<Vec<_>>();

        assert!(depth_results.iter().any(|result| matches!(
        result,
        Err(failure)
            if matches!(failure.kind, WorkspaceDiscoveryFailureKind::DepthLimitExceeded { limit: 1 })
    )));
        let mut count_request = options(&workspace);
        count_request.limits.files = 1;
        let count_results = session()
            .discover_workspace_documents(count_request)
            .collect::<Vec<_>>();
        assert_eq!(successful_uris(&count_results).len(), 1);
        assert!(count_results.iter().any(|result| matches!(
        result,
        Err(failure)
            if matches!(failure.kind, WorkspaceDiscoveryFailureKind::FileLimitExceeded { limit: 1 })
    )));
    }

    #[test]
    fn sparse_trees_stop_at_the_visited_entry_bound() {
        let workspace = TestWorkspace::new("entry-bound");
        workspace.write("a.txt", "unsupported");
        workspace.write("b.txt", "unsupported");
        workspace.write("c/package.json", "{}");
        let mut request = options(&workspace);
        request.limits.visited_entries = 1;

        let results = session()
            .discover_workspace_documents(request)
            .collect::<Vec<_>>();

        assert!(results.iter().any(|result| matches!(
            result,
            Err(failure)
                if failure.kind
                    == WorkspaceDiscoveryFailureKind::EntryLimitExceeded { limit: 1 }
        )));
        assert!(results.len() <= 2);
    }

    #[test]
    fn cancellation_is_one_terminal_outcome() {
        let workspace = TestWorkspace::new("cancel");
        workspace.write("package.json", "{}");
        let request = options(&workspace);
        let cancellation = request.cancellation.clone();
        cancellation.cancel();
        let mut discovery = session().discover_workspace_documents(request);

        assert!(matches!(
            discovery.next(),
            Some(Err(failure)) if failure.kind == WorkspaceDiscoveryFailureKind::Cancelled
        ));
        assert!(discovery.next().is_none());
    }

    #[test]
    fn missing_roots_are_reported_with_the_failed_operation() {
        let workspace = TestWorkspace::new("missing-root");
        let missing = workspace.root.join("missing");
        let results = session()
            .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![missing.clone()]))
            .collect::<Vec<_>>();

        assert!(matches!(
            results.as_slice(),
            [Err(failure)]
                if failure.path.as_deref() == Some(Path::new(&missing))
                    && matches!(
                        failure.kind,
                        WorkspaceDiscoveryFailureKind::Io {
                            operation: WorkspaceDiscoveryIoOperation::ResolvePath,
                            ..
                        }
                    )
        ));
    }
}
