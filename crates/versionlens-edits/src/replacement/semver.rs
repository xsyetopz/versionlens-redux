pub(super) fn semver_selector_latest<'a>(prefix: &str, latest: &'a str) -> &'a str {
    if prefix.ends_with("semver:") {
        latest.strip_prefix('v').unwrap_or(latest)
    } else {
        latest
    }
}

pub(super) fn preserve_semver_range_prefix(requirement: &str, latest: &str) -> String {
    let leading = requirement
        .chars()
        .take_while(|char| !char.is_ascii_digit())
        .collect::<String>();

    match leading.as_str() {
        "^" | "~" | ">" | ">=" | "~>" | "==" => {
            format!("{leading}{latest}")
        }
        _ => latest.to_owned(),
    }
}
