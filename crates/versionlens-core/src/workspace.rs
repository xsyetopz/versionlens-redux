//! Conservative workspace/source-of-truth discovery.
//!
//! Only explicit workspace declarations are traversed. All candidates are
//! bounded by the opened workspace root and malformed or duplicate identities
//! are left unresolved instead of being guessed.
mod discovery;
mod edits;
mod paths;
pub use paths::{workspace_file_uri, workspace_path};

use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use semver::{Version, VersionReq};
use serde_json::Value;
use versionlens_model::{
    Dependency, DocumentEditPlan, DocumentInput, DocumentSnapshot, Ecosystem, TextEdit,
    WorkspaceEditPlan, document_text_hash,
};

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

pub(crate) type WorkspaceDocuments = BTreeMap<PathBuf, (String, Option<u64>)>;

#[derive(Debug, Clone, Default)]
pub(crate) struct WorkspaceGraph {
    pub(crate) fingerprint: Option<versionlens_cache::CacheKey>,
    root: Option<PathBuf>,
    source: Option<PathBuf>,
    members: Arc<BTreeMap<(Ecosystem, String), (String, PathBuf)>>,
    duplicates: Arc<BTreeSet<(Ecosystem, String)>>,
    policy: WorkspacePolicy,
    pnpm_default_external: bool,
    documents: Arc<WorkspaceDocuments>,
}

impl WorkspaceGraph {
    pub(crate) fn for_workspace(root: &Path, overlays: &WorkspaceDocuments) -> Self {
        let read = |path: &Path| {
            overlays
                .get(path)
                .map(|(text, _)| text.clone())
                .or_else(|| fs::read_to_string(path).ok())
        };
        let pnpm_default_external = read(&root.join("package.json"))
            .and_then(|text| serde_json::from_str::<Value>(&text).ok())
            .and_then(|value| {
                value
                    .get("packageManager")
                    .and_then(Value::as_str)
                    .map(|value| value.starts_with("pnpm@"))
            })
            .unwrap_or(false);
        let inherited_cargo_version =
            read(&root.join("Cargo.toml")).and_then(|text| cargo_workspace_version(&text));
        let mut members = BTreeMap::new();
        let mut duplicates = BTreeSet::new();
        let mut documents = WorkspaceDocuments::new();
        let overlay_paths = overlays.keys().cloned().collect::<Vec<_>>();
        let manifests = discovery::manifests(root, &read, &overlay_paths);
        for manifest in manifests {
            let Some(text) = read(&manifest) else {
                continue;
            };
            let version = overlays.get(&manifest).and_then(|(_, version)| *version);
            let identity = identity(&manifest, &text, inherited_cargo_version.as_deref());
            documents.insert(manifest.clone(), (text, version));
            let Some((name, version, ecosystem)) = identity else {
                continue;
            };
            let key = (ecosystem, name);
            if duplicates.contains(&key) {
                continue;
            }
            match members.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert((version, manifest));
                }
                Entry::Occupied(entry) => {
                    let (key, _) = entry.remove_entry();
                    duplicates.insert(key);
                }
            }
        }
        let policy = lerna_policy(read(&root.join("lerna.json")).as_deref());
        let identity = format!("{members:?}|{duplicates:?}|{policy:?}|{pnpm_default_external}");
        Self {
            fingerprint: Some(versionlens_cache::CacheKey::content(identity.as_bytes())),
            root: Some(root.to_path_buf()),
            source: None,
            members: Arc::new(members),
            duplicates: Arc::new(duplicates),
            policy,
            pnpm_default_external,
            documents: Arc::new(documents),
        }
    }

    pub(crate) fn bind_source(&self, input: &DocumentInput) -> Self {
        Self {
            source: document_path(input),
            ..self.clone()
        }
    }

    pub(crate) fn captures(&self, input: &DocumentInput) -> bool {
        document_path(input)
            .and_then(|path| self.documents.get(&path))
            .is_some_and(|(text, version)| {
                text == &input.text && version.is_none_or(|version| Some(version) == input.version)
            })
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
        changes.insert(document_path(input).ok_or(())?, active_edits.to_vec());

        let governed = |name: &str| self.policy == WorkspacePolicy::Fixed || name == selected_name;
        for ((ecosystem, name), (_, manifest)) in self.members.iter() {
            if *ecosystem != Ecosystem::Npm {
                continue;
            }
            let (text, _) = self.documents.get(manifest).ok_or(())?;
            let edits = edits::package_edits(
                text,
                selected_name,
                &target_version,
                selected_version,
                governed(name),
            );
            if !edits.is_empty() {
                changes.entry(manifest.clone()).or_default().extend(edits);
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
            let (text, version) = if document_path(input) == Some(path.clone()) {
                (&input.text, input.version)
            } else {
                let (text, version) = self.documents.get(&path).ok_or(())?;
                (text, *version)
            };
            documents.push(DocumentEditPlan {
                document: DocumentSnapshot {
                    uri,
                    version,
                    text_hash: document_text_hash(text),
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
    let Some(root) = graph.root.as_deref() else {
        return LocalPathClassification::Invalid;
    };
    let Some(candidate) = discovery::bounded(root, &source.join(relative)) else {
        return LocalPathClassification::Invalid;
    };
    let manifest = if candidate.is_file() {
        candidate
    } else if dependency.ecosystem == Ecosystem::Cargo {
        candidate.join("Cargo.toml")
    } else {
        candidate.join("package.json")
    };
    let Some(manifest) = discovery::bounded(root, &manifest) else {
        return LocalPathClassification::Invalid;
    };
    if graph.documents.contains_key(&manifest) {
        LocalPathClassification::Valid(manifest)
    } else {
        LocalPathClassification::Invalid
    }
}

pub(crate) fn workspace_root(input: &DocumentInput) -> Option<PathBuf> {
    input
        .workspace_root
        .as_deref()
        .and_then(workspace_path)?
        .canonicalize()
        .ok()
}

pub(crate) fn document_path(input: &DocumentInput) -> Option<PathBuf> {
    let declared_root = input.workspace_root.as_deref().and_then(workspace_path)?;
    let root = declared_root.canonicalize().ok()?;
    let declared_path = workspace_path(&input.uri)?;
    let path = declared_path
        .strip_prefix(&declared_root)
        .map_or(declared_path.clone(), |relative| root.join(relative));
    discovery::bounded(&root, &path)
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
fn identity(
    path: &Path,
    text: &str,
    inherited_cargo_version: Option<&str>,
) -> Option<(String, String, Ecosystem)> {
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
            let document = text.parse::<toml_edit::DocumentMut>().ok()?;
            let package = document.get("package")?;
            let version = package.get("version")?;
            let version = version.as_str().map(str::to_owned).or_else(|| {
                version
                    .as_table_like()?
                    .get("workspace")?
                    .as_bool()
                    .filter(|inherited| *inherited)
                    .and(inherited_cargo_version.map(str::to_owned))
            })?;
            Some((
                package.get("name")?.as_str()?.to_owned(),
                version,
                Ecosystem::Cargo,
            ))
        }
        _ => None,
    }
}

fn cargo_workspace_version(text: &str) -> Option<String> {
    text.parse::<toml_edit::DocumentMut>()
        .ok()?
        .get("workspace")?
        .get("package")?
        .get("version")?
        .as_str()
        .map(str::to_owned)
}
fn lerna_policy(text: Option<&str>) -> WorkspacePolicy {
    let Some(v) = text.and_then(|text| serde_json::from_str::<Value>(text).ok()) else {
        return WorkspacePolicy::None;
    };
    match v.get("version").and_then(Value::as_str) {
        Some("independent") => WorkspacePolicy::Independent,
        Some(_) => WorkspacePolicy::Fixed,
        None => WorkspacePolicy::None,
    }
}

#[cfg(test)]
pub(crate) mod tests;
