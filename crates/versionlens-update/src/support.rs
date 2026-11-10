pub(crate) fn normalize_version_part(part: &str) -> Option<String> {
    if part == "*" || part.eq_ignore_ascii_case("x") {
        return Some("0".to_owned());
    }
    part.chars()
        .all(|character| character.is_ascii_digit())
        .then(|| part.to_owned())
}

pub(crate) fn normalize_version_token(token: &str) -> Option<String> {
    let (core, prerelease) = token
        .split_once('-')
        .map_or((token, None), |(core, prerelease)| (core, Some(prerelease)));
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let mut normalized = parts
        .into_iter()
        .map(normalize_version_part)
        .collect::<Option<Vec<_>>>()?;
    normalized.resize(3, "0".to_owned());
    let core = normalized.join(".");
    Some(match prerelease {
        Some(suffix) => format!("{core}-{suffix}"),
        None => core,
    })
}

pub(crate) fn minimum_version_token(requirement: &str) -> Option<&str> {
    let first_range = requirement
        .trim()
        .split("||")
        .next()?
        .split(',')
        .next()?
        .trim();
    let token = first_range
        .trim_start_matches(['^', '~', '>', '<', '=', 'v', 'V'])
        .split_whitespace()
        .next()?;
    (!token.is_empty()).then_some(token)
}

#[cfg(test)]
pub(crate) mod tests {
    pub(crate) fn empty_range() -> versionlens_model::Range {
        versionlens_model::Range {
            start: versionlens_model::Position {
                line: 0,
                character: 0,
            },
            end: versionlens_model::Position {
                line: 0,
                character: 0,
            },
        }
    }

    pub(crate) fn test_dependency(name: &str, requirement: &str) -> versionlens_model::Dependency {
        versionlens_model::Dependency {
            name: name.to_owned(),
            requirement: requirement.to_owned(),
            ecosystem: versionlens_model::Ecosystem::Cargo,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: empty_range(),
            requirement_range: empty_range(),
            requirement_prefix: String::new(),
            requirement_suffix: String::new(),
            canonical_reference: None,
        }
    }

    pub(crate) fn fixture(base: &str, name: &str) -> &'static str {
        versionlens_test_support::static_fixture!(base, name)
            .expect("test fixture must be readable")
    }

    pub(crate) fn range(
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
    ) -> versionlens_model::Range {
        versionlens_model::Range {
            start: versionlens_model::Position {
                line: start_line,
                character: start_character,
            },
            end: versionlens_model::Position {
                line: end_line,
                character: end_character,
            },
        }
    }
}
