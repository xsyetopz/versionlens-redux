use versionlens_model::{DocumentInput, ManifestKind, TextEdit};

pub(super) fn package_edits(
    text: &str,
    name: &str,
    previous: &str,
    selected: &str,
    update_project: bool,
) -> Vec<TextEdit> {
    let input = DocumentInput::new("package.json", "json", text, None);
    versionlens_parsers::parse_document_as_manifest_with_dependency_paths::<&str>(
        &input,
        ManifestKind::NpmPackageJson,
        &[],
    )
    .into_iter()
    .filter_map(|dependency| {
        let new_text = if dependency.group == "version" && update_project {
            selected.to_owned()
        } else {
            if dependency.name != name
                || dependency.is_runtime_version()
                || dependency.requirement.starts_with("workspace:")
                || dependency.requirement.starts_with("catalog:")
                || dependency.hosted_url.is_some()
            {
                return None;
            }
            replace_version(&dependency.requirement, previous, selected)?
        };
        Some(TextEdit {
            range: dependency.requirement_range,
            new_text,
        })
    })
    .collect()
}

fn replace_version(requirement: &str, previous: &str, selected: &str) -> Option<String> {
    if previous.is_empty() {
        return None;
    }
    let mut replacement = String::new();
    let mut offset = 0;
    for (start, _) in requirement.match_indices(previous) {
        let end = start + previous.len();
        let before = &requirement[..start];
        let before = before.strip_suffix('v').unwrap_or(before);
        if before.chars().next_back().is_some_and(version_character)
            || requirement[end..]
                .chars()
                .next()
                .is_some_and(version_character)
        {
            continue;
        }
        replacement.push_str(&requirement[offset..start]);
        replacement.push_str(selected);
        offset = end;
    }
    if offset == 0 {
        return None;
    }
    replacement.push_str(&requirement[offset..]);
    Some(replacement)
}

fn version_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '.' | '-' | '+')
}

#[cfg(test)]
mod tests;
