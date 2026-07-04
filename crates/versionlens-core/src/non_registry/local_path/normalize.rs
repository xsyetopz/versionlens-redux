use std::path::{Component, Path, PathBuf};

pub(super) fn resolve_local_path(path: &str, document_uri: Option<&str>) -> String {
    let path = Path::new(path);
    let Some(parent) = document_uri
        .and_then(|uri| uri.strip_prefix("file://"))
        .and_then(|path| Path::new(path).parent())
        .filter(|_| path.is_relative())
    else {
        return path.to_string_lossy().into_owned();
    };

    normalize_path(parent.join(path))
        .to_string_lossy()
        .into_owned()
}

fn normalize_path(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        apply_path_component(&mut normalized, component);
    }
    normalized
}

fn apply_path_component(normalized: &mut PathBuf, component: Component<'_>) {
    match component {
        Component::CurDir => {}
        Component::ParentDir => {
            normalized.pop();
        }
        _ => normalized.push(component.as_os_str()),
    }
}
