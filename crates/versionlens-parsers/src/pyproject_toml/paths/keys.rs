use toml_edit::Key;

pub(in crate::pyproject_toml) struct TomlPathContext {
    pub(in crate::pyproject_toml) path: String,
    pub(in crate::pyproject_toml) parent: String,
}

pub(in crate::pyproject_toml) fn paths_for_keys(keys: &[&Key]) -> TomlPathContext {
    TomlPathContext {
        path: key_path(keys),
        parent: parent_path(keys),
    }
}

pub(in crate::pyproject_toml::paths) fn parent_path(keys: &[&Key]) -> String {
    keys[..keys.len().saturating_sub(1)]
        .iter()
        .map(|key| key.get())
        .collect::<Vec<_>>()
        .join(".")
}

fn key_path(keys: &[&Key]) -> String {
    keys.iter()
        .map(|key| key.get())
        .collect::<Vec<_>>()
        .join(".")
}
