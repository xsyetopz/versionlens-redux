pub(crate) fn path_or_member_enabled(patterns: &[&str], group: &str, name: Option<&str>) -> bool {
    patterns.iter().any(|pattern| {
        path_matches(pattern, group)
            || name.is_some_and(|name| {
                let member = format!("{group}.{name}");
                path_matches(pattern, &member)
            })
    })
}

pub(crate) fn path_or_member_enabled_exact(
    patterns: &[&str],
    group: &str,
    name: Option<&str>,
) -> bool {
    patterns.iter().any(|pattern| {
        path_matches_exact(pattern, group)
            || name.is_some_and(|name| {
                let member = format!("{group}.{name}");
                path_matches_exact(pattern, &member)
            })
    })
}

fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == path || path.starts_with(&format!("{pattern}.")) {
        return true;
    }

    path_matches_exact(pattern, path)
}

fn path_matches_exact(pattern: &str, path: &str) -> bool {
    let pattern_segments = pattern.split('.').collect::<Vec<_>>();
    let path_segments = path.split('.').collect::<Vec<_>>();
    pattern_segments.len() == path_segments.len()
        && pattern_segments
            .iter()
            .zip(path_segments)
            .all(|(pattern, path)| *pattern == "*" || *pattern == path)
}
