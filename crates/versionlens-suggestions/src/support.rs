use semver::{Error as SemverError, Version as SemverVersion};

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

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
pub(crate) mod tests;
