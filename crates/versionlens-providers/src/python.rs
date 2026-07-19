pub(crate) fn normalized_project_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut in_separator = false;

    for character in name.chars() {
        if matches!(character, '-' | '_' | '.') {
            in_separator = true;
            continue;
        }
        if in_separator && !normalized.is_empty() {
            normalized.push('-');
        }
        in_separator = false;
        normalized.extend(character.to_lowercase());
    }

    normalized
}
