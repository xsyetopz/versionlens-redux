use toml_edit::Key;

const CARGO_DEPENDENCY_PATHS: &[&str] = &[
    "package",
    "dependencies",
    "dependencies.*",
    "dev-dependencies",
    "dev-dependencies.*",
    "build-dependencies",
    "build-dependencies.*",
    "workspace.dependencies",
    "workspace.dependencies.*",
];

pub(super) fn selected_dependency_paths<'a>(dependency_paths: &'a [&'a str]) -> Vec<&'a str> {
    if dependency_paths.is_empty() {
        CARGO_DEPENDENCY_PATHS.to_vec()
    } else {
        dependency_paths.to_vec()
    }
}

pub(super) fn is_cargo_project_version(keys: &[&Key], dependency_paths: &[&str]) -> bool {
    keys.len() == 2
        && keys[0].get() == "package"
        && keys[1].get() == "version"
        && dependency_paths.contains(&"package")
}

pub(super) fn match_cargo_dependency_table<'a>(
    keys: &[&Key],
    dependency_paths: &'a [&str],
) -> Option<&'a str> {
    if keys.len() < 2 {
        return None;
    }

    let table_segments = keys[..keys.len() - 1]
        .iter()
        .map(|key| key.get())
        .collect::<Vec<_>>();

    for include_path in dependency_paths {
        if table_path_matches(&table_segments, include_path) {
            return Some(*include_path);
        }
    }

    None
}

fn table_path_matches(table_segments: &[&str], include_path: &str) -> bool {
    let include_segments = include_path.split('.').collect::<Vec<_>>();
    segments_match(table_segments, &include_segments)
        || terminal_wildcard_matches(table_segments, &include_segments)
}

fn terminal_wildcard_matches(table_segments: &[&str], include_segments: &[&str]) -> bool {
    let Some((last, prefix_segments)) = include_segments.split_last() else {
        return false;
    };
    if *last != "*" || table_segments.len() <= prefix_segments.len() {
        return false;
    }

    segments_match(&table_segments[..prefix_segments.len()], prefix_segments)
}

fn segments_match(table_segments: &[&str], include_segments: &[&str]) -> bool {
    table_segments.len() == include_segments.len()
        && table_segments
            .iter()
            .zip(include_segments)
            .all(|(table, include)| *include == "*" || table == include)
}
