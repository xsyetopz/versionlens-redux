use std::path::Component::{CurDir as PathCurDir, ParentDir as PathParentDir};
use std::path::{Component, PathBuf};

pub(super) fn resolve_local_path(path: &str, document_uri: Option<&str>) -> String {
    let path = crate::path(path);
    let Some(parent) = document_uri
        .and_then(|uri| uri.strip_prefix("file://"))
        .and_then(|path| crate::path(path).parent())
        .filter(|_| path.is_relative())
    else {
        return path.to_string_lossy().into_owned();
    };

    normalize_path(parent.join(path))
        .to_string_lossy()
        .into_owned()
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = crate::default();
    for component in path.components() {
        apply_path_component(&mut normalized, component);
    }
    normalized
}

fn apply_path_component(normalized: &mut PathBuf, component: Component<'_>) {
    match component {
        PathCurDir => {}
        PathParentDir => {
            normalized.pop();
        }
        _ => normalized.push(component.as_os_str()),
    }
}
