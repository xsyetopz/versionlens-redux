use std::collections::BTreeSet;

pub(super) fn dependency_groups<'a>(dependency_paths: &[&'a str]) -> Vec<&'a str> {
    let mut seen: BTreeSet<&str> = crate::default();
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
