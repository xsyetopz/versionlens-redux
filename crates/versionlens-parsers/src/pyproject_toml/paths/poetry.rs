use toml_edit::Key;

use super::keys::parent_path;

pub(in crate::pyproject_toml) fn is_poetry_dependency_path(keys: &[&Key]) -> bool {
    let path = parent_path(keys);
    path == "tool.poetry.dependencies"
        || path == "tool.poetry.dev-dependencies"
        || (path.starts_with("tool.poetry.group.") && path.ends_with(".dependencies"))
}
