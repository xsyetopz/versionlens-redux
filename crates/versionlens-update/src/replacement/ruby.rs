pub(super) fn ruby_replacement(requirement: &str, latest: &str) -> String {
    let Some(operator) = leading_ruby_operator(requirement) else {
        return latest.to_owned();
    };

    let replacement_operator = match operator.trim_end() {
        "<" | "<=" | "!=" => "==",
        operator => operator,
    };
    let whitespace = &operator[operator.trim_end().len()..];
    format!("{replacement_operator}{whitespace}{latest}")
}

pub(super) fn ruby_prefixed_replacement(prefix: &str, suffix: &str, latest: &str) -> String {
    let prefix = if is_git_sha(latest) {
        ref_option_prefix(prefix).unwrap_or_else(|| prefix.to_owned())
    } else {
        prefix.to_owned()
    };

    format!("{prefix}{latest}{suffix}")
}

fn leading_ruby_operator(version: &str) -> Option<&str> {
    const OPERATORS: [&str; 7] = ["~>", ">=", ">", "<=", "<", "==", "!="];

    OPERATORS.iter().find_map(|operator| {
        let rest = version.strip_prefix(operator)?;
        let whitespace_len = rest
            .char_indices()
            .take_while(|(_, char)| char.is_whitespace())
            .map(|(index, char)| index + char.len_utf8())
            .last()
            .unwrap_or(0);
        Some(&version[..operator.len() + whitespace_len])
    })
}

fn is_git_sha(version: &str) -> bool {
    !version.contains('.')
        && version.len() >= 7
        && version.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

fn ref_option_prefix(prefix: &str) -> Option<String> {
    let option_start = prefix
        .char_indices()
        .find_map(|(index, char)| (!char.is_whitespace()).then_some(index))
        .unwrap_or(prefix.len());
    let leading = &prefix[..option_start];
    let option = &prefix[option_start..];

    for attr in ["tag", "branch"] {
        let Some(tail) = option.strip_prefix(attr) else {
            continue;
        };
        if tail.trim_start().starts_with(':') {
            return Some(format!("{leading}ref{tail}"));
        }
    }

    None
}
