pub(crate) fn normalize_requirement(requirement: &str) -> String {
    let parts = requirement
        .replacen("~> ", "~", 1)
        .replacen("~>", "~", 1)
        .split_whitespace()
        .filter(|part| !part.eq_ignore_ascii_case("and"))
        .map(normalize_requirement_part)
        .collect::<Vec<_>>();

    join_spaced_comparators(&parts)
}

fn join_spaced_comparators(parts: &[String]) -> String {
    let mut output = vec![];
    let mut index = 0;
    while index < parts.len() {
        let part = parts[index].as_str();
        if matches!(part, ">" | ">=" | "<" | "<=" | "=" | "==")
            && let Some(version) = parts.get(index + 1)
        {
            output.push(format!("{part}{version}"));
            index += 2;
            continue;
        }

        output.push(part.to_owned());
        index += 1;
    }

    output.join(" ")
}
pub fn strip_version_prefix(part: &str) -> String {
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
