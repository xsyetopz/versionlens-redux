pub(super) fn property_matches(pattern: &str, path: &str) -> bool {
    pattern == path
        || path.starts_with(&format!("{pattern}."))
        || glob_segments_match(
            &pattern.split('.').collect::<Vec<_>>(),
            &path.split('.').collect::<Vec<_>>(),
        )
}

fn glob_segments_match(pattern: &[&str], path: &[&str]) -> bool {
    pattern.len() == path.len()
        && pattern
            .iter()
            .zip(path)
            .all(|(left, right)| *left == "*" || left == right)
}
