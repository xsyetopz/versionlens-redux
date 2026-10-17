pub(super) const PACKAGE_EXTENSION_GROUPS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];

pub(super) fn root_dependency_group(path: &str) -> Option<&str> {
    if path.split_once('.').is_some() || matches!(path, "catalog" | "overrides") {
        None
    } else {
        Some(path)
    }
}
