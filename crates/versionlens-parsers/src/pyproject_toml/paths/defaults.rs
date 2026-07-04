const PYPI_DEPENDENCY_PATHS: &[&str] = &[
    "project",
    "packages",
    "dev-packages",
    "tool.poetry.dependencies",
    "tool.poetry.group.*.dependencies",
    "project.optional-dependencies",
];

pub(in crate::pyproject_toml) fn selected_dependency_paths<'a>(
    dependency_paths: &'a [&'a str],
) -> Vec<&'a str> {
    if dependency_paths.is_empty() {
        PYPI_DEPENDENCY_PATHS.to_vec()
    } else {
        dependency_paths.to_vec()
    }
}
