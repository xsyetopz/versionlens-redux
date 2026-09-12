use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};

use super::{
    WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryIoOperation,
};

pub(super) enum ReadFailure {
    Io(io::Error),
    TooLarge(u64),
    InvalidUtf8,
}

pub(super) fn read_bounded_utf8(path: &Path, limit: u64) -> Result<String, ReadFailure> {
    let file = File::open(path).map_err(ReadFailure::Io)?;
    let read_limit = limit.saturating_add(1);
    let mut bytes = Vec::new();
    file.take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(ReadFailure::Io)?;
    let size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if size > limit {
        return Err(ReadFailure::TooLarge(size));
    }
    String::from_utf8(bytes).map_err(|_| ReadFailure::InvalidUtf8)
}

pub(super) fn verify_overlay_containment(
    path: &Path,
    root: &Path,
) -> Result<(), WorkspaceDiscoveryFailure> {
    let mut existing = path;
    loop {
        match fs::symlink_metadata(existing) {
            Ok(_) => {
                let canonical = fs::canonicalize(existing).map_err(|error| {
                    WorkspaceDiscoveryFailure::io(
                        path.to_path_buf(),
                        WorkspaceDiscoveryIoOperation::ResolvePath,
                        &error,
                    )
                })?;
                return if canonical.starts_with(root) {
                    Ok(())
                } else {
                    Err(WorkspaceDiscoveryFailure::at(
                        path.to_path_buf(),
                        WorkspaceDiscoveryFailureKind::EscapesWorkspace,
                    ))
                };
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(parent) = existing.parent() else {
                    return Err(WorkspaceDiscoveryFailure::io(
                        path.to_path_buf(),
                        WorkspaceDiscoveryIoOperation::ResolvePath,
                        &error,
                    ));
                };
                existing = parent;
            }
            Err(error) => {
                return Err(WorkspaceDiscoveryFailure::io(
                    path.to_path_buf(),
                    WorkspaceDiscoveryIoOperation::ReadMetadata,
                    &error,
                ));
            }
        }
    }
}

pub(super) fn normalize_absolute(path: &Path) -> Option<PathBuf> {
    if !path.is_absolute() {
        return None;
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
        }
    }
    Some(normalized)
}

pub fn slash_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn exclusion_matches(pattern: &str, path: &str, directory: bool) -> bool {
    let pattern = pattern
        .trim()
        .trim_start_matches("./")
        .trim_start_matches('/');
    if pattern.is_empty() {
        return false;
    }
    let mut candidates = Vec::with_capacity(2);
    candidates.push(path.to_owned());
    if directory {
        candidates.push(format!("{path}/"));
    }
    candidates.into_iter().any(|candidate| {
        path_glob_matches(pattern, &candidate)
            || (!pattern.contains('/')
                && candidate
                    .trim_end_matches('/')
                    .split('/')
                    .any(|segment| glob_matches(pattern.as_bytes(), segment.as_bytes())))
    })
}

fn path_glob_matches(pattern: &str, path: &str) -> bool {
    if glob_matches(pattern.as_bytes(), path.as_bytes()) {
        return true;
    }
    let Some(suffix) = pattern.strip_prefix("**/") else {
        return false;
    };
    glob_matches(suffix.as_bytes(), path.as_bytes())
        || path
            .match_indices('/')
            .any(|(index, _)| glob_matches(suffix.as_bytes(), &path.as_bytes()[index + 1..]))
}

fn glob_matches(pattern: &[u8], text: &[u8]) -> bool {
    let width = text.len() + 1;
    let mut previous = vec![false; width];
    previous[0] = true;
    let mut index = 0;
    while index < pattern.len() {
        let double_star = pattern[index] == b'*' && pattern.get(index + 1) == Some(&b'*');
        let token = pattern[index];
        index += if double_star { 2 } else { 1 };
        let mut current = vec![false; width];
        if token == b'*' {
            current[0] = previous[0];
            for text_index in 1..width {
                let character = text[text_index - 1];
                current[text_index] = previous[text_index]
                    || current[text_index - 1] && (double_star || character != b'/');
            }
        } else {
            for text_index in 1..width {
                let character = text[text_index - 1];
                current[text_index] = previous[text_index - 1]
                    && (token == character || token == b'?' && character != b'/');
            }
        }
        previous = current;
    }
    previous[text.len()]
}

pub(super) fn language_id(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    match extension.to_ascii_lowercase().as_str() {
        "json" => "json",
        "json5" => "json5",
        "jsonc" => "jsonc",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        "xml" | "csproj" | "fsproj" | "vbproj" | "props" | "targets" => "xml",
        "gradle" | "kts" => "gradle",
        "dockerfile" => "dockerfile",
        _ if name.eq_ignore_ascii_case("Dockerfile") => "dockerfile",
        _ => "plaintext",
    }
    .to_owned()
}
