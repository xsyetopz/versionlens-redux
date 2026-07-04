pub(super) const PACKAGE_EXTENSION_GROUPS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

const PNPM_DEPENDENCY_PATHS: &[&str] = &[
    "catalog",
    "catalogs.*.*",
    "overrides",
    "packageExtensions.*.dependencies",
    "packageExtensions.*.devDependencies",
    "packageExtensions.*.peerDependencies",
    "packageExtensions.*.optionalDependencies",
];

pub(super) fn selected_dependency_paths<'a>(dependency_paths: &'a [&'a str]) -> Vec<&'a str> {
    if dependency_paths.is_empty() {
        PNPM_DEPENDENCY_PATHS.to_vec()
    } else {
        dependency_paths.to_vec()
    }
}

pub(super) fn root_dependency_group(path: &str) -> Option<&str> {
    if path.split_once('.').is_some() || matches!(path, "catalog" | "overrides") {
        None
    } else {
        Some(path)
    }
}
