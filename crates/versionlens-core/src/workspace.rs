//! Conservative workspace/source-of-truth discovery.
//!
//! Only explicit workspace declarations are traversed. All candidates are
//! bounded by the opened workspace root and malformed or duplicate identities
//! are left unresolved instead of being guessed.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use semver::{Version, VersionReq};
use serde_json::Value;
use versionlens_model::{
    Dependency, DocumentEditPlan, DocumentInput, DocumentSnapshot, Ecosystem, Position, Range,
    TextEdit, WorkspaceEditPlan, document_text_hash,
};

const MAX_DISCOVERY_NODES: usize = 4096;
type WorkspacePaths = Vec<PathBuf>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum WorkspacePolicy {
    #[default]
    None,
    Fixed,
    Independent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalResolution {
    pub(crate) version: String,
    pub(crate) policy: WorkspacePolicy,
    pub(crate) manifest: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalPathClassification {
    NotPath,
    Valid(PathBuf),
    Invalid,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceGraph {
    root: Option<PathBuf>,
    source: Option<PathBuf>,
    members: BTreeMap<(Ecosystem, String), (String, PathBuf)>,
    duplicates: BTreeSet<(Ecosystem, String)>,
    policy: WorkspacePolicy,
    pnpm_default_external: bool,
}

impl WorkspaceGraph {
    pub(crate) fn for_document(input: &DocumentInput) -> Self {
        let Some(root) = input.workspace_root.as_deref().and_then(workspace_path) else {
            return Self::default();
        };
        let Ok(root) = root.canonicalize() else {
            return Self::default();
        };
        let pnpm_default_external = fs::read_to_string(root.join("package.json"))
            .ok()
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|value| {
                value
                    .get("packageManager")
                    .and_then(Value::as_str)
                    .map(|value| value.starts_with("pnpm@"))
            })
            .unwrap_or(false);
        let mut graph = Self {
            root: Some(root.clone()),
            source: workspace_path(&input.uri).and_then(|path| path.canonicalize().ok()),
            policy: lerna_policy(&root),
            pnpm_default_external,
            ..Self::default()
        };
        let mut manifests = BTreeSet::new();
        let package = root.join("package.json");
        if package.is_file() {
            manifests.extend(package_members(&root, &package));
        }
        let pnpm = root.join("pnpm-workspace.yaml");
        if pnpm.is_file() {
            manifests.extend(pnpm_members(&root, &pnpm));
        }
        let cargo = root.join("Cargo.toml");
        if cargo.is_file() {
            manifests.extend(cargo_members(&root, &cargo));
        }
        let current = workspace_path(&input.uri).and_then(|p| p.canonicalize().ok());
        for manifest in manifests {
            let text = if current.as_deref() == Some(manifest.as_path()) {
                Some(input.text.clone())
            } else {
                fs::read_to_string(&manifest).ok()
            };
            let Some(text) = text else { continue };
            let Some((name, version, ecosystem)) = identity(&manifest, &text) else {
                continue;
            };
            let key = (ecosystem, name);
            if graph.members.contains_key(&key) {
                graph.members.remove(&key);
                graph.duplicates.insert(key);
            } else if !graph.duplicates.contains(&key) {
                graph.members.insert(key, (version, manifest.clone()));
            }
        }
        graph
    }

    pub(crate) fn resolve(&self, dependency: &Dependency) -> Option<LocalResolution> {
        if dependency.requirement.trim().starts_with("catalog:") {
            // A catalog is a manager-owned indirection. Never mistake a
            // same-named workspace member for the catalog source.
            return None;
        }
        let target_manifest = local_target_manifest(self, dependency);
        if matches!(target_manifest, LocalPathClassification::Invalid) {
            return None;
        }
        let key = (dependency.ecosystem, dependency.name.clone());
        if self.duplicates.contains(&key) {
            return None;
        }
        let (version, manifest) = self.members.get(&key)?;
        if let LocalPathClassification::Valid(target) = target_manifest {
            if target != *manifest {
                return None;
            }
        }
        if dependency.ecosystem == Ecosystem::Npm
            && self.pnpm_default_external
            && !local_reference(&dependency.requirement)
        {
            return None;
        }
        if dependency.ecosystem == Ecosystem::Npm && !local_reference(&dependency.requirement) {
            let range = VersionReq::parse(dependency.requirement.trim()).ok()?;
            if !range.matches(&Version::parse(version).ok()?) {
                return None;
            }
        }
        Some(LocalResolution {
            version: version.clone(),
            policy: self.policy,
            manifest: manifest.clone(),
        })
    }

    /// Builds the complete edit set for a proven local package update.  The
    /// planner deliberately works from manifest identity and reverse edges,
    /// rather than from registry suggestions, so a same-named external package
    /// can never cause unrelated files to be edited.
    pub(crate) fn coordinated_plan(
        &self,
        input: &DocumentInput,
        active_edits: &[TextEdit],
        selected_name: &str,
        selected_version: &str,
    ) -> Result<Option<WorkspaceEditPlan>, ()> {
        if self.members.is_empty() {
            return Ok(None);
        }
        let target = self
            .members
            .get(&(Ecosystem::Npm, selected_name.to_owned()))
            .ok_or(())?;
        let target_version = target.0.clone();
        let mut changes: BTreeMap<PathBuf, Vec<TextEdit>> = BTreeMap::new();
        changes.insert(
            canonical_input_path(input).ok_or(())?,
            active_edits.to_vec(),
        );

        let governed = |name: &str| self.policy == WorkspacePolicy::Fixed || name == selected_name;
        for ((ecosystem, name), (_, manifest)) in &self.members {
            if *ecosystem != Ecosystem::Npm {
                continue;
            }
            let text = fs::read_to_string(manifest).map_err(|_| ())?;
            if governed(name) {
                if let Some(edit) = json_string_edit(&text, "version", selected_version) {
                    changes.entry(manifest.clone()).or_default().push(edit);
                }
            }
            let refs =
                package_dependency_edits(&text, selected_name, &target_version, selected_version);
            if !refs.is_empty() {
                changes.entry(manifest.clone()).or_default().extend(refs);
            }
        }
        let mut documents = Vec::new();
        for (path, mut edits) in changes {
            edits.sort_by_key(|edit| (edit.range.start.line, edit.range.start.character));
            edits.dedup_by(|left, right| {
                left.range == right.range && left.new_text == right.new_text
            });
            if edits.windows(2).any(|pair| overlap(&pair[0], &pair[1])) {
                return Err(());
            }
            let uri = path.to_string_lossy().into_owned();
            let text = if canonical_input_path(input) == Some(path.clone()) {
                input.text.clone()
            } else {
                fs::read_to_string(&path).map_err(|_| ())?
            };
            documents.push(DocumentEditPlan {
                document: DocumentSnapshot {
                    uri,
                    version: if canonical_input_path(input) == Some(path.clone()) {
                        input.version
                    } else {
                        None
                    },
                    text_hash: document_text_hash(&text),
                },
                edits,
            });
        }
        let plan = WorkspaceEditPlan { documents };
        let current = plan
            .documents
            .iter()
            .map(|doc| {
                (
                    doc.document.uri.clone(),
                    doc.document.version,
                    doc.document.text_hash.clone(),
                )
            })
            .collect::<Vec<_>>();
        crate::validate_workspace_edit_plan(&plan, &current).map_err(|_| ())?;
        Ok(Some(plan))
    }
}

fn local_target_manifest(
    graph: &WorkspaceGraph,
    dependency: &Dependency,
) -> LocalPathClassification {
    let value = dependency.requirement.trim();
    let relative = value
        .strip_prefix("file:")
        .or_else(|| value.strip_prefix("link:"))
        .or_else(|| value.strip_prefix("path:"))
        .or_else(|| value.starts_with("./").then_some(value))
        .or_else(|| value.starts_with("../").then_some(value));
    let Some(relative) = relative else {
        return LocalPathClassification::NotPath;
    };
    if relative.is_empty() {
        return LocalPathClassification::Invalid;
    }
    let Some(source) = graph.source.as_deref().and_then(Path::parent) else {
        return LocalPathClassification::Invalid;
    };
    let candidate = source.join(relative);
    let Ok(candidate) = candidate.canonicalize() else {
        return LocalPathClassification::Invalid;
    };
    let Some(root) = graph.root.as_deref() else {
        return LocalPathClassification::Invalid;
    };
    if !candidate.starts_with(root) {
        return LocalPathClassification::Invalid;
    }
    let manifest = if candidate.is_file() {
        candidate
    } else if dependency.ecosystem == Ecosystem::Cargo {
        candidate.join("Cargo.toml")
    } else {
        candidate.join("package.json")
    };
    match manifest.canonicalize() {
        Ok(manifest) if manifest.starts_with(root) && manifest.is_file() => {
            LocalPathClassification::Valid(manifest)
        }
        _ => LocalPathClassification::Invalid,
    }
}

fn canonical_input_path(input: &DocumentInput) -> Option<PathBuf> {
    workspace_path(&input.uri)?.canonicalize().ok()
}

fn json_string_edit(text: &str, key: &str, value: &str) -> Option<TextEdit> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)?;
    let colon = text[start + needle.len()..].find(':')? + start + needle.len();
    let quote = text[colon + 1..].find('"')? + colon + 1;
    let end = text[quote + 1..].find('"')? + quote + 1;
    Some(TextEdit {
        range: text_range(text, quote + 1, end),
        new_text: value.to_owned(),
    })
}

fn package_dependency_edits(text: &str, name: &str, old: &str, new: &str) -> Vec<TextEdit> {
    let mut edits = Vec::new();
    let key = format!("\"{name}\"");
    let mut offset = 0;
    while let Some(found) = text[offset..].find(&key) {
        let start = offset + found;
        let after = &text[start + key.len()..];
        let Some(colon) = after.find(':') else { break };
        let value_start = start + key.len() + colon + 1;
        let Some(open) = text[value_start..].find('"') else {
            break;
        };
        let quote = value_start + open;
        let Some(close) = text[quote + 1..].find('"') else {
            break;
        };
        let end = quote + 1 + close;
        let value = &text[quote + 1..end];
        if value.starts_with("workspace:") || value.starts_with("catalog:") {
            offset = end + 1;
            continue;
        }
        if value == old || value.contains(old) {
            let replacement = value.replace(old, new);
            edits.push(TextEdit {
                range: text_range(text, quote + 1, end),
                new_text: replacement,
            });
        }
        offset = end + 1;
    }
    edits
}

fn text_range(text: &str, start: usize, end: usize) -> Range {
    fn position(text: &str, offset: usize) -> Position {
        let prefix = &text[..offset];
        Position {
            line: u32::try_from(prefix.bytes().filter(|byte| *byte == b'\n').count())
                .unwrap_or(u32::MAX),
            character: u32::try_from(prefix.rsplit('\n').next().unwrap_or_default().len())
                .unwrap_or(u32::MAX),
        }
    }
    Range {
        start: position(text, start),
        end: position(text, end),
    }
}

fn overlap(left: &TextEdit, right: &TextEdit) -> bool {
    (left.range.start.line < right.range.end.line
        || (left.range.start.line == right.range.end.line
            && left.range.start.character < right.range.end.character))
        && (right.range.start.line < left.range.end.line
            || (right.range.start.line == left.range.end.line
                && right.range.start.character < left.range.end.character))
}

fn local_reference(value: &str) -> bool {
    let v = value.trim();
    [
        "workspace:",
        "catalog:",
        "file:",
        "link:",
        "path:",
        "./",
        "../",
    ]
    .iter()
    .any(|p| v.starts_with(p))
}
fn workspace_path(value: &str) -> Option<PathBuf> {
    path_from_uri(value).or_else(|| (!value.is_empty()).then(|| PathBuf::from(value)))
}
fn path_from_uri(value: &str) -> Option<PathBuf> {
    Some(PathBuf::from(
        value.strip_prefix("file://")?.replace("%20", " "),
    ))
}
fn bounded(root: &Path, path: PathBuf) -> Option<PathBuf> {
    let path = path.canonicalize().ok()?;
    path.starts_with(root).then_some(path)
}

fn package_members(root: &Path, manifest: &Path) -> WorkspacePaths {
    let Ok(value) =
        serde_json::from_str::<Value>(&fs::read_to_string(manifest).ok().unwrap_or_default())
    else {
        return vec![];
    };
    let patterns = match value.get("workspaces") {
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).collect::<Vec<_>>(),
        Some(Value::Object(o)) => o
            .get("packages")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default(),
        _ => vec![],
    };
    let mut result = vec![manifest.to_path_buf()];
    for pattern in patterns {
        result.extend(expand_package_pattern(root, pattern));
    }
    result
}
fn pnpm_members(root: &Path, manifest: &Path) -> WorkspacePaths {
    let Ok(text) = fs::read_to_string(manifest) else {
        return vec![];
    };
    text.lines()
        .filter_map(|line| {
            let value = line
                .trim()
                .strip_prefix("- ")?
                .trim()
                .trim_matches(['\'', '"']);
            (!value.starts_with('!')).then(|| expand_package_pattern(root, value))
        })
        .flatten()
        .collect()
}
fn cargo_members(root: &Path, manifest: &Path) -> WorkspacePaths {
    let Ok(text) = fs::read_to_string(manifest) else {
        return vec![];
    };
    let mut workspace = false;
    let mut result = vec![manifest.to_path_buf()];
    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            workspace = t == "[workspace]";
            continue;
        }
        if workspace && t.starts_with("members") {
            if let Some(v) = t.split_once('=').map(|(_, v)| v) {
                result.extend(
                    v.trim()
                        .trim_matches(['[', ']'])
                        .split(',')
                        .filter_map(|p| {
                            bounded(root, root.join(p.trim().trim_matches(['\'', '"'])))
                                .map(|p| p.join("Cargo.toml"))
                        }),
                );
            }
        }
    }
    result
}
fn expand_package_pattern(root: &Path, pattern: &str) -> WorkspacePaths {
    if !pattern.contains('*') {
        let p = root.join(pattern);
        let m = p.join("package.json");
        return m.is_file().then_some(m).into_iter().collect();
    }
    let mut dirs = vec![root.to_path_buf()];
    for part in pattern.trim_end_matches('/').split('/') {
        let mut next = vec![];
        for dir in dirs {
            if part.contains('*') {
                if let Ok(entries) = fs::read_dir(dir) {
                    next.extend(
                        entries
                            .flatten()
                            .take(MAX_DISCOVERY_NODES)
                            .map(|e| e.path())
                            .filter(|p| p.is_dir() && !p.ends_with("node_modules")),
                    );
                }
            } else {
                next.push(dir.join(part));
            }
        }
        dirs = next;
    }
    dirs.into_iter()
        .map(|d| d.join("package.json"))
        .take(MAX_DISCOVERY_NODES)
        .filter(|p| bounded(root, p.clone()).is_some() && p.is_file())
        .collect()
}
fn identity(path: &Path, text: &str) -> Option<(String, String, Ecosystem)> {
    match path.file_name()?.to_str()? {
        "package.json" => {
            let v = serde_json::from_str::<Value>(text).ok()?;
            Some((
                v.get("name")?.as_str()?.into(),
                v.get("version")?.as_str()?.into(),
                Ecosystem::Npm,
            ))
        }
        "Cargo.toml" => {
            let s = text.split("[package]").nth(1)?;
            Some((
                toml_field(s, "name")?,
                toml_field(s, "version")?,
                Ecosystem::Cargo,
            ))
        }
        _ => None,
    }
}
fn toml_field(section: &str, field: &str) -> Option<String> {
    section
        .lines()
        .find_map(|l| l.trim().strip_prefix(&format!("{field} =")))
        .map(|v| v.trim().trim_matches(['\'', '"']).into())
        .filter(|v: &String| !v.is_empty())
}
fn lerna_policy(root: &Path) -> WorkspacePolicy {
    let Ok(v) = serde_json::from_str::<Value>(
        &fs::read_to_string(root.join("lerna.json"))
            .ok()
            .unwrap_or_default(),
    ) else {
        return WorkspacePolicy::None;
    };
    match v.get("version").and_then(Value::as_str) {
        Some("independent") => WorkspacePolicy::Independent,
        Some(_) => WorkspacePolicy::Fixed,
        None => WorkspacePolicy::None,
    }
}

#[cfg(test)]
mod tests;
