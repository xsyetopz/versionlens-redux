use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use marked_yaml::parse_yaml;
use marked_yaml::types::Node::{Scalar as YamlScalar, Sequence as YamlSequence};
use serde_json::Value as JsonValue;
use toml_edit::{DocumentMut as TomlDocument, Item as TomlItem};

use self::matcher::WorkspacePattern;

mod matcher;

const PACKAGE_MANIFEST: &str = "package.json";
const CARGO_MANIFEST: &str = "Cargo.toml";
const IGNORED_DIRECTORIES: &[&str] = &[".git", "node_modules", "target", "vendor", "build", "dist"];

/// Finds manifests selected by workspace declarations below a canonical root.
///
/// `read` is the source of truth for file contents and may provide unsaved
/// overlays. `overlays` lets a matching manifest participate before it exists
/// on disk; paths outside `root` are always rejected.
pub(super) fn manifests(
    root: &Path,
    read: &impl Fn(&Path) -> Option<String>,
    overlays: &[PathBuf],
) -> Vec<PathBuf> {
    let mut result = BTreeSet::new();

    if let Some(package) = bounded(root, &root.join(PACKAGE_MANIFEST)) {
        if let Some(text) = read(&package) {
            result.insert(package);
            if let Some(patterns) = package_patterns(&text) {
                result.extend(expand(root, PACKAGE_MANIFEST, &patterns, read, overlays));
            }
        }
    }

    if let Some(pnpm) = bounded(root, &root.join("pnpm-workspace.yaml")) {
        if let Some(text) = read(&pnpm)
            && let Some(patterns) = pnpm_patterns(&text)
        {
            result.extend(expand(root, PACKAGE_MANIFEST, &patterns, read, overlays));
        }
    }

    if let Some(cargo) = bounded(root, &root.join(CARGO_MANIFEST)) {
        if let Some(text) = read(&cargo) {
            result.insert(cargo);
            if let Some(patterns) = cargo_patterns(&text) {
                result.extend(expand(root, CARGO_MANIFEST, &patterns, read, overlays));
            }
        }
    }

    result.into_iter().collect()
}

#[derive(Debug)]
struct Patterns {
    include: Vec<WorkspacePattern>,
    exclude: Vec<WorkspacePattern>,
}

struct DiskCandidateSearch<'a> {
    root: &'a Path,
    manifest_name: &'a str,
    patterns: &'a Patterns,
    explicitly_named_ignored: BTreeSet<&'static str>,
}

impl DiskCandidateSearch<'_> {
    fn ignores_directory(&self, name: &str) -> bool {
        IGNORED_DIRECTORIES.contains(&name) && !self.explicitly_named_ignored.contains(name)
    }

    fn collect(&self, directory: &Path, result: &mut BTreeSet<PathBuf>) {
        let relative = directory.strip_prefix(self.root).unwrap_or(Path::new(""));
        if !relative.as_os_str().is_empty() && selected(self.patterns, relative) {
            let manifest = directory.join(self.manifest_name);
            if let Some(manifest) = bounded(self.root, &manifest)
                && manifest.is_file()
            {
                result.insert(manifest);
            }
        }

        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let mut directories = entries
            .flatten()
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                file_type.is_dir().then_some(entry.path())
            })
            .collect::<Vec<_>>();
        directories.sort();
        for child in directories {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if self.ignores_directory(name) {
                continue;
            }
            let Ok(relative) = child.strip_prefix(self.root) else {
                continue;
            };
            let components = path_segments(relative);
            if self
                .patterns
                .include
                .iter()
                .any(|pattern| pattern.could_match_descendant(&components))
            {
                self.collect(&child, result);
            }
        }
    }
}

impl Patterns {
    fn from_signed(values: impl IntoIterator<Item = String>) -> Self {
        let mut include = Vec::new();
        let mut exclude = Vec::new();
        for value in values {
            if value.starts_with("!(") {
                if let Some(pattern) = WorkspacePattern::parse(&value) {
                    include.push(pattern);
                }
            } else if let Some(value) = value.strip_prefix('!') {
                if let Some(pattern) = WorkspacePattern::parse(value) {
                    exclude.push(pattern);
                }
            } else if let Some(pattern) = WorkspacePattern::parse(&value) {
                include.push(pattern);
            }
        }
        Self { include, exclude }
    }
}

fn package_patterns(text: &str) -> Option<Patterns> {
    let value = serde_json::from_str::<JsonValue>(text).ok()?;
    let workspaces = value.get("workspaces")?;
    let values = match workspaces {
        JsonValue::Array(values) => string_json_values(values),
        JsonValue::Object(value) => value
            .get("packages")
            .and_then(JsonValue::as_array)
            .map_or_else(Vec::new, |values| string_json_values(values)),
        _ => return None,
    };
    Some(Patterns::from_signed(values))
}

fn string_json_values(values: &[JsonValue]) -> Vec<String> {
    values
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_owned)
        .collect()
}

fn pnpm_patterns(text: &str) -> Option<Patterns> {
    let document = parse_yaml(0, text).ok()?;
    let root = document.as_mapping()?;
    let YamlSequence(values) = root.get_node("packages")? else {
        return None;
    };
    let values = values
        .iter()
        .filter_map(|value| {
            let YamlScalar(value) = value else {
                return None;
            };
            Some(value.as_str().to_owned())
        })
        .collect::<Vec<_>>();
    Some(Patterns::from_signed(values))
}

fn cargo_patterns(text: &str) -> Option<Patterns> {
    let document = text.parse::<TomlDocument>().ok()?;
    let workspace = document.get("workspace")?.as_table_like()?;
    let include = toml_strings(workspace.get("members"));
    let exclude = toml_strings(workspace.get("exclude"));
    Some(Patterns {
        include: include
            .iter()
            .filter_map(|value| WorkspacePattern::parse(value))
            .collect(),
        exclude: exclude
            .iter()
            .filter_map(|value| WorkspacePattern::parse(value))
            .collect(),
    })
}

fn toml_strings(item: Option<&TomlItem>) -> Vec<String> {
    item.and_then(TomlItem::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(toml_edit::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn expand(
    root: &Path,
    manifest_name: &str,
    patterns: &Patterns,
    read: &impl Fn(&Path) -> Option<String>,
    overlays: &[PathBuf],
) -> BTreeSet<PathBuf> {
    if patterns.include.is_empty() {
        return BTreeSet::new();
    }

    let search = DiskCandidateSearch {
        root,
        manifest_name,
        patterns,
        explicitly_named_ignored: IGNORED_DIRECTORIES
            .iter()
            .copied()
            .filter(|name| {
                patterns
                    .include
                    .iter()
                    .any(|pattern| pattern.explicitly_names(name))
            })
            .collect(),
    };
    let mut candidates = BTreeSet::new();
    search.collect(root, &mut candidates);
    for overlay in overlays {
        if overlay.file_name().and_then(|name| name.to_str()) != Some(manifest_name) {
            continue;
        }
        let Some(path) = bounded(root, overlay) else {
            continue;
        };
        let Some(relative) = overlay
            .parent()
            .and_then(|parent| parent.strip_prefix(root).ok())
        else {
            continue;
        };
        let relative_segments = path_segments(relative);
        if relative_segments
            .iter()
            .any(|name| search.ignores_directory(name))
        {
            continue;
        }
        if selected(patterns, relative) && read(&path).is_some() {
            candidates.insert(path);
        }
    }
    candidates
}

fn selected(patterns: &Patterns, relative: &Path) -> bool {
    let path = path_segments(relative);
    patterns
        .include
        .iter()
        .any(|pattern| pattern.matches(&path))
        && !patterns
            .exclude
            .iter()
            .any(|pattern| pattern.matches(&path))
}

fn path_segments(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect()
}

pub(super) fn bounded(root: &Path, path: &Path) -> Option<PathBuf> {
    let path = lexical_normalize(path);
    if !path.starts_with(root) {
        return None;
    }
    let mut ancestor = path.as_path();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => {
                let canonical = ancestor.canonicalize().ok()?;
                if !canonical.starts_with(root) {
                    return None;
                }
                if ancestor != path && !canonical.is_dir() {
                    return None;
                }
                return if ancestor == path {
                    Some(canonical)
                } else {
                    Some(path)
                };
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor.parent()?;
            }
            Err(_) => return None,
        }
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

#[cfg(test)]
mod tests;
