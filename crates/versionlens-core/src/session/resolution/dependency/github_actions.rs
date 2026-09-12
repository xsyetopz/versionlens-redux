use std::borrow::Cow;
use std::fs;
use std::path::{Component, Path};

use versionlens_model::{CanonicalReference, Dependency};
use versionlens_suggestions::{Suggestion, error, fixed};

use crate::workspace::WorkspaceDocuments;

pub(super) fn github_action_reference_suggestion(
    dependency: Dependency,
    workspace_root: Option<&Path>,
    workspace_documents: &WorkspaceDocuments,
) -> Option<Suggestion> {
    match dependency.canonical_reference.as_ref()? {
        CanonicalReference::GitHubActionExpression { .. } => Some(error(
            dependency,
            "GitHub action reference expression cannot be resolved statically".to_owned(),
        )),
        CanonicalReference::GitHubActionInvalid { .. } => Some(error(
            dependency,
            "invalid GitHub action reference".to_owned(),
        )),
        CanonicalReference::GitHubActionLocal {
            path,
            reusable_workflow,
        } => Some(local_github_action_suggestion(
            dependency.clone(),
            workspace_root,
            workspace_documents,
            path,
            *reusable_workflow,
        )),
        CanonicalReference::GitHubActionTag { .. }
        | CanonicalReference::GitHubActionSha { .. }
        | CanonicalReference::GitHubActionCommit { .. }
        | CanonicalReference::GitHubActionRef { .. } => None,
    }
}

fn local_github_action_suggestion(
    dependency: Dependency,
    workspace_root: Option<&Path>,
    workspace_documents: &WorkspaceDocuments,
    path: &str,
    reusable_workflow: bool,
) -> Suggestion {
    let Some(workspace_root) = workspace_root else {
        return error(
            dependency,
            "workspace root unavailable for local GitHub action reference".to_owned(),
        );
    };
    let Some(relative) = path.strip_prefix("./").map(Path::new) else {
        return error(dependency, "invalid local GitHub action path".to_owned());
    };
    if relative.as_os_str().is_empty()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return error(dependency, "invalid local GitHub action path".to_owned());
    }
    if reusable_workflow
        && (!path.starts_with("./.github/workflows/")
            || relative.components().count() != 3
            || !matches!(
                relative.extension().and_then(|value| value.to_str()),
                Some("yml" | "yaml")
            ))
    {
        return error(
            dependency,
            "invalid local reusable workflow path".to_owned(),
        );
    }

    let Ok(canonical_root) = workspace_root.canonicalize() else {
        return error(
            dependency,
            "workspace root unavailable for local GitHub action reference".to_owned(),
        );
    };
    let target = canonical_root.join(relative);
    if reusable_workflow {
        return local_reusable_workflow_suggestion(
            dependency,
            workspace_documents,
            &canonical_root,
            &target,
            path,
        );
    }
    local_action_manifest_suggestion(
        dependency,
        workspace_documents,
        &canonical_root,
        &target,
        path,
    )
}

fn local_reusable_workflow_suggestion(
    dependency: Dependency,
    workspace_documents: &WorkspaceDocuments,
    workspace_root: &Path,
    target: &Path,
    display_path: &str,
) -> Suggestion {
    let Some(text) = workspace_file_text(workspace_documents, workspace_root, target) else {
        return error(
            dependency,
            format!("local reusable workflow `{display_path}` is unavailable"),
        );
    };
    if reusable_workflow_is_valid(&text) {
        fixed(dependency, display_path.to_owned())
    } else {
        error(
            dependency,
            format!("local reusable workflow `{display_path}` is invalid"),
        )
    }
}

fn local_action_manifest_suggestion(
    dependency: Dependency,
    workspace_documents: &WorkspaceDocuments,
    workspace_root: &Path,
    target: &Path,
    display_path: &str,
) -> Suggestion {
    let manifests = ["action.yml", "action.yaml"]
        .into_iter()
        .filter_map(|name| {
            workspace_file_text(workspace_documents, workspace_root, &target.join(name))
        })
        .collect::<Vec<_>>();
    match manifests.as_slice() {
        [manifest] if action_manifest_is_valid(manifest) => {
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

fn action_manifest_is_valid(text: &str) -> bool {
    let Ok(document) = marked_yaml::parse_yaml(0, text) else {
        return false;
    };
    document
        .as_mapping()
        .and_then(|root| root.get_mapping("runs"))
        .and_then(|runs| runs.get_scalar("using"))
        .is_some_and(|using| !using.as_str().trim().is_empty())
}

fn reusable_workflow_is_valid(text: &str) -> bool {
    let Ok(document) = marked_yaml::parse_yaml(0, text) else {
        return false;
    };
    let Some(root) = document.as_mapping() else {
        return false;
    };
    if root
        .get_scalar("on")
        .is_some_and(|event| event.as_str() == "workflow_call")
    {
        return true;
    }
    if root
        .get_mapping("on")
        .is_some_and(|events| events.get_node("workflow_call").is_some())
    {
        return true;
    }
    root.get_sequence("on").is_some_and(|events| {
        events.iter().any(|event| {
            event
                .as_scalar()
                .is_some_and(|event| event.as_str() == "workflow_call")
        })
    })
}
