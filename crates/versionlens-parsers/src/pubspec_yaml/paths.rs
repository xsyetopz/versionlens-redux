use std::collections::BTreeSet;

pub(super) const PUBSPEC_DEPENDENCY_PATHS: &[&str] = &[
    "version",
    "dependencies",
    "dev_dependencies",
    "dependency_overrides",
];

pub(super) fn selected_dependency_paths<'a>(dependency_paths: &'a [&'a str]) -> Vec<&'a str> {
    if dependency_paths.is_empty() {
        PUBSPEC_DEPENDENCY_PATHS.to_vec()
    } else {
        dependency_paths.to_vec()
    }
}

pub(super) fn dependency_groups<'a>(dependency_paths: &[&'a str]) -> Vec<&'a str> {
    let mut seen = BTreeSet::new();
    dependency_paths
        .iter()
        .filter_map(|path| dependency_group(path))
        .filter(|group| seen.insert(*group))
        .collect()
}

fn dependency_group<'a>(path: &'a str) -> Option<&'a str> {
    let group = path.split_once('.').map_or(path, |(group, _)| group);
    (group != "version").then_some(group)
}
