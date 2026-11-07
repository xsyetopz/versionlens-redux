use std::borrow::Cow;
use std::fs;
use std::ops::ControlFlow;
use std::path::{Component, Path};

use versionlens_model::{CanonicalReference, Dependency};
use versionlens_update::{Suggestion, error, fixed};

use crate::workspace::WorkspaceDocuments;

/// `Continue` leaves the dependency to the registry. `Break(None)` gives it no
/// result at all, because its local target is ignored or excluded.
pub(super) fn github_action_reference_suggestion(
    dependency: Dependency,
    workspace_root: Option<&Path>,
    workspace_documents: &WorkspaceDocuments,
    skips: &mut impl FnMut(&Path, &Path) -> bool,
) -> ControlFlow<Option<Suggestion>> {
    let Some(reference) = dependency.canonical_reference.as_ref() else {
        return ControlFlow::Continue(());
    };
    match reference {
        CanonicalReference::GitHubActionExpression { .. } => ControlFlow::Break(Some(error(
            dependency,
            "GitHub action reference expression cannot be resolved statically".to_owned(),
        ))),
        CanonicalReference::GitHubActionInvalid { .. } => ControlFlow::Break(Some(error(
            dependency,
            "invalid GitHub action reference".to_owned(),
        ))),
        CanonicalReference::GitHubActionLocal {
            path,
            reusable_workflow,
        } => ControlFlow::Break(local_github_action_suggestion(
            dependency.clone(),
            workspace_root,
            workspace_documents,
            skips,
            path,
            *reusable_workflow,
        )),
        // A branch or non-version tag names no release to compare, so it is
        // shown as a fixed ref without a registry request.
        CanonicalReference::GitHubActionRef { reference } => {
            let reference = reference.clone();
            ControlFlow::Break(Some(fixed(dependency, reference)))
        }
        CanonicalReference::GitHubActionTag { .. }
        | CanonicalReference::GitHubActionSha { .. }
        | CanonicalReference::GitHubActionCommit { .. } => ControlFlow::Continue(()),
    }
}

fn local_github_action_suggestion(
    dependency: Dependency,
    workspace_root: Option<&Path>,
    workspace_documents: &WorkspaceDocuments,
    skips: &mut impl FnMut(&Path, &Path) -> bool,
    path: &str,
    reusable_workflow: bool,
) -> Option<Suggestion> {
    let Some(workspace_root) = workspace_root else {
        return Some(error(
            dependency,
            "workspace root unavailable for local GitHub action reference".to_owned(),
        ));
    };
    let Some(relative) = path.strip_prefix("./").map(Path::new) else {
        return Some(error(
            dependency,
            "invalid local GitHub action path".to_owned(),
        ));
    };
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Some(error(
            dependency,
            "invalid local GitHub action path".to_owned(),
        ));
    }
    if reusable_workflow
        && (!path.starts_with("./.github/workflows/")
            || relative.components().count() != 3
            || !matches!(
                relative.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            ))
    {
        return Some(error(
            dependency,
            "invalid local reusable workflow path".to_owned(),
        ));
    }

    let Ok(canonical_root) = workspace_root.canonicalize() else {
        return Some(error(
            dependency,
            "workspace root unavailable for local GitHub action reference".to_owned(),
        ));
    };
    let target = canonical_root.join(relative);
    // An ignored or excluded target is neither read nor reported on, like the
    // ignored files themselves. A path skipped next to a readable one counts
    // as missing.
    let files = if reusable_workflow {
        vec![target.clone()]
    } else {
        vec![target.join("action.yml"), target.join("action.yaml")]
    };
    if files.iter().all(|file| skips(&canonical_root, file)) {
        return None;
    }
    let mut read = |path: &Path| {
        if skips(&canonical_root, path) {
            return None;
        }
        workspace_file_text(workspace_documents, &canonical_root, path)
    };
    if reusable_workflow {
        return Some(local_reusable_workflow_suggestion(
            dependency, &mut read, &target, path,
        ));
    }
    Some(local_action_manifest_suggestion(
        dependency, &mut read, &target, path,
    ))
}

fn local_reusable_workflow_suggestion<'a>(
    dependency: Dependency,
    read: &mut impl FnMut(&Path) -> Option<Cow<'a, str>>,
    target: &Path,
    display_path: &str,
) -> Suggestion {
    let Some(text) = read(target) else {
        return error(
            dependency,
            format!("local reusable workflow `{display_path}` is unavailable"),
        );
    };
    if versionlens_parsers::reusable_workflow_is_valid(&text) {
        fixed(dependency, display_path.to_owned())
    } else {
        error(
            dependency,
            format!("local reusable workflow `{display_path}` is invalid"),
        )
    }
}

fn local_action_manifest_suggestion<'a>(
    dependency: Dependency,
    read: &mut impl FnMut(&Path) -> Option<Cow<'a, str>>,
    target: &Path,
    display_path: &str,
) -> Suggestion {
    let manifests = ["action.yml", "action.yaml"]
        .into_iter()
        .filter_map(|name| read(&target.join(name)))
        .collect::<Vec<_>>();
    match manifests.as_slice() {
        [manifest] if versionlens_parsers::action_manifest_is_valid(manifest) => {
            fixed(dependency, display_path.to_owned())
        }
        [_] => error(
            dependency,
            format!("local GitHub action path `{display_path}` is invalid"),
        ),
        [] => error(
            dependency,
            format!("local GitHub action path `{display_path}` is unavailable"),
        ),
        _ => error(
            dependency,
            format!("local GitHub action path `{display_path}` is ambiguous"),
        ),
    }
}

fn workspace_file_text<'a>(
    workspace_documents: &'a WorkspaceDocuments,
    workspace_root: &Path,
    path: &Path,
) -> Option<Cow<'a, str>> {
    if let Some((text, _)) = workspace_documents.get(path) {
        return Some(Cow::Borrowed(text));
    }
    let canonical = path.canonicalize().ok()?;
    if !canonical.starts_with(workspace_root) || !canonical.is_file() {
        return None;
    }
    fs::read_to_string(canonical).ok().map(Cow::Owned)
}
