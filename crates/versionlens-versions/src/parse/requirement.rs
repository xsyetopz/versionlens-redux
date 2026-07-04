pub(crate) fn normalize_requirement(requirement: &str) -> String {
    requirement
        .replacen("~> ", "~", 1)
        .replacen("~>", "~", 1)
        .split_whitespace()
        .map(normalize_requirement_part)
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn strip_version_prefix(part: &str) -> String {
    let split_at = part
        .find(|char: char| char.is_ascii_digit() || matches!(char, 'v' | 'V'))
        .unwrap_or(part.len());
    let (prefix, version) = part.split_at(split_at);
    if let Some(rest) = version
        .strip_prefix('v')
        .or_else(|| version.strip_prefix('V'))
    {
        return format!("{prefix}{rest}");
    }

    part.to_owned()
}

fn normalize_requirement_part(part: &str) -> String {
    if part.eq_ignore_ascii_case("any") {
        return "*".to_owned();
    }

    strip_version_prefix(part)
}
