use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use versionlens_model::DocumentInput;

use crate::workspace::{WorkspaceDocuments, document_path, workspace_path};

const MAX_REGISTRY_FILE_BYTES: usize = 1024 * 1024;
const MAX_REGISTRY_SNAPSHOT_BYTES: usize = 4 * 1024 * 1024;
const MAX_REGISTRY_SNAPSHOT_ENTRIES: usize = 4096;

pub(crate) trait RegistryFileRead {
    fn read(&self, path: &Path) -> Option<String>;

    fn read_required(&self, path: &Path) -> Option<String>;

    fn failure(&self) -> Option<RegistryFileFailure>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RegistryFileFailure {
    message: String,
}

impl RegistryFileFailure {
    fn new(path: &Path, detail: impl std::fmt::Display) -> Self {
        Self {
            message: format!(
                "failed to load registry configuration {}: {detail}",
                path.display()
            ),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RegistryFileSnapshot {
    documents: Arc<WorkspaceDocuments>,
    disk: RegistryDiskSnapshot,
}

impl RegistryFileSnapshot {
    pub(crate) fn new(documents: Arc<WorkspaceDocuments>) -> Self {
        Self {
            documents,
            disk: crate::default(),
        }
    }

    pub(crate) fn reader<'a>(&'a self, input: &'a DocumentInput) -> RegistryFileReader<'a> {
        let declared_root = input.workspace_root.as_deref().and_then(workspace_path);
        let canonical_root = declared_root
            .as_ref()
            .and_then(|root| root.canonicalize().ok());
        RegistryFileReader {
            snapshot: self,
            active: input.workspace_root.as_ref().and_then(|_| {
                document_path(input)
                    .or_else(|| workspace_path(&input.uri))
                    .map(|path| (path, input.text.as_str()))
            }),
            declared_root,
            canonical_root,
            failure: Mutex::new(None),
        }
    }

    fn read_overlay(path: &Path, text: &str) -> RegistryDiskEntry {
        if text.len() > MAX_REGISTRY_FILE_BYTES {
            return Err(RegistryFileFailure::new(
                path,
                format_args!(
                    "file is {} bytes; the limit is {MAX_REGISTRY_FILE_BYTES} bytes",
                    text.len()
                ),
            ));
        }
        Ok(Some(text.to_owned()))
    }

    fn read(&self, path: &Path) -> RegistryDiskEntry {
        if let Some((text, _)) = self.documents.get(path) {
            return Self::read_overlay(path, text);
        }
        self.disk.read(path)
    }
}

pub(crate) struct RegistryFileReader<'a> {
    snapshot: &'a RegistryFileSnapshot,
    active: Option<(PathBuf, &'a str)>,
    declared_root: Option<PathBuf>,
    canonical_root: Option<PathBuf>,
    failure: Mutex<Option<RegistryFileFailure>>,
}

impl RegistryFileRead for RegistryFileReader<'_> {
    fn read(&self, path: &Path) -> Option<String> {
        self.read_inner(path, false)
    }

    fn read_required(&self, path: &Path) -> Option<String> {
        self.read_inner(path, true)
    }

    fn failure(&self) -> Option<RegistryFileFailure> {
        self.failure
            .lock()
            .unwrap_or_else(crate::recover_poison)
            .clone()
    }
}

impl RegistryFileReader<'_> {
    fn read_inner(&self, path: &Path, required: bool) -> Option<String> {
        if self.failure().is_some() {
            return None;
        }
        let path = self.normalized(path);
        let result =
            if let Some((_, text)) = self.active.as_ref().filter(|(active, _)| active == &path) {
                RegistryFileSnapshot::read_overlay(&path, text)
            } else {
                self.snapshot.read(&path)
            };
        match result {
            Ok(Some(text)) => Some(text),
            Ok(None) if required => {
                self.fail(RegistryFileFailure::new(
                    &path,
                    "required file was not found",
                ));
                None
            }
            Ok(None) => None,
            Err(failure) => {
                self.fail(failure);
                None
            }
        }
    }

    fn fail(&self, failure: RegistryFileFailure) {
        self.failure
            .lock()
            .unwrap_or_else(crate::recover_poison)
            .get_or_insert(failure);
    }

    fn normalized(&self, path: &Path) -> PathBuf {
        self.declared_root
            .as_ref()
            .zip(self.canonical_root.as_ref())
            .and_then(|(declared, canonical)| {
                path.strip_prefix(declared)
                    .ok()
                    .map(|relative| canonical.join(relative))
            })
            .unwrap_or_else(|| path.to_path_buf())
    }
}

#[derive(Debug, Clone, Default)]
struct RegistryDiskSnapshot {
    state: Arc<Mutex<RegistryDiskSnapshotState>>,
}

impl RegistryDiskSnapshot {
    fn read(&self, path: &Path) -> RegistryDiskEntry {
        let mut state = self.state.lock().unwrap_or_else(crate::recover_poison);
        if let Some(value) = state.files.get(path) {
            return value.clone();
        }
        if state.files.len() >= MAX_REGISTRY_SNAPSHOT_ENTRIES {
            return Err(RegistryFileFailure::new(
                path,
                format_args!(
                    "snapshot contains {MAX_REGISTRY_SNAPSHOT_ENTRIES} files; no more files can be read in this workspace generation"
                ),
            ));
        }

        let value = bounded_read_to_string(path);
        let value_bytes = registry_entry_bytes(path, &value);
        if state.bytes.saturating_add(value_bytes) > MAX_REGISTRY_SNAPSHOT_BYTES {
            return Err(RegistryFileFailure::new(
                path,
                format_args!(
                    "snapshot contents and path metadata exceed the {MAX_REGISTRY_SNAPSHOT_BYTES}-byte workspace generation limit"
                ),
            ));
        }
        state.bytes += value_bytes;
        state.files.insert(path.to_path_buf(), value.clone());
        value
    }
}

type RegistryDiskEntry = Result<Option<String>, RegistryFileFailure>;

#[derive(Debug, Default)]
struct RegistryDiskSnapshotState {
    files: BTreeMap<PathBuf, RegistryDiskEntry>,
    bytes: usize,
}

fn registry_entry_bytes(path: &Path, value: &RegistryDiskEntry) -> usize {
    let payload = match value {
        Ok(Some(text)) => text.capacity(),
        Ok(None) => 0,
        Err(failure) => failure.message.capacity(),
    };
    std::mem::size_of::<PathBuf>()
        + std::mem::size_of::<RegistryDiskEntry>()
        + 3 * std::mem::size_of::<usize>()
        + path.as_os_str().len()
        + payload
}

fn bounded_read_to_string(path: &Path) -> Result<Option<String>, RegistryFileFailure> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(RegistryFileFailure::new(path, error)),
    };
    let size = file
        .metadata()
        .map_err(|error| RegistryFileFailure::new(path, error))?
        .len();
    if size > MAX_REGISTRY_FILE_BYTES as u64 {
        return Err(RegistryFileFailure::new(
            path,
            format_args!("file is {size} bytes; the limit is {MAX_REGISTRY_FILE_BYTES} bytes"),
        ));
    }

    let capacity = usize::try_from(size).map_err(|error| RegistryFileFailure::new(path, error))?;
    let mut bytes = Vec::with_capacity(capacity);
    file.take(MAX_REGISTRY_FILE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| RegistryFileFailure::new(path, error))?;
    if bytes.len() > MAX_REGISTRY_FILE_BYTES {
        return Err(RegistryFileFailure::new(
            path,
            format_args!("file exceeds the {MAX_REGISTRY_FILE_BYTES}-byte limit while being read"),
        ));
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|error| RegistryFileFailure::new(path, error))
}

#[cfg(test)]
mod tests;
