const GITHUB_URL_PREFIXES: &[&str] = &[
    "git+https://github.com/",
    "git+ssh://git@github.com/",
    "git+ssh://git@github.com:",
    "git+ssh://github.com/",
    "git+ssh://github.com:",
    "https://github.com/",
    "http://github.com/",
    "git://github.com/",
    "ssh://git@github.com/",
    "ssh://git@github.com:",
    "ssh://github.com/",
    "ssh://github.com:",
    "git@github.com:",
];

pub(super) fn github_dependency_name(value: &str) -> Option<&str> {
    if let Some(name) = strip_prefix_ignore_case(value, "github:") {
        return normalize_github_name(name);
    }
    if bare_github_shortcut(value) {
        return normalize_github_name(value);
    }

    GITHUB_URL_PREFIXES
        .iter()
        .find_map(|prefix| strip_prefix_ignore_case(value, prefix))
        .and_then(normalize_github_name)
}

fn bare_github_shortcut(value: &str) -> bool {
    !value.contains(':') && !value.contains("://")
}

fn normalize_github_name(value: &str) -> Option<&str> {
    let value = value.strip_suffix(".git").unwrap_or(value);
    let (owner, repo) = value.split_once('/')?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') {
        return None;
    }
    Some(value)
}

fn strip_prefix_ignore_case<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
        .then(|| &value[prefix.len()..])
}
