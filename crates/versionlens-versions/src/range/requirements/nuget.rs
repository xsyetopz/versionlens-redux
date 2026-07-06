use crate::parse::{parse_padded_version, parse_version};

pub(in crate::range) fn nuget_requirement_satisfies(
    requirement: &str,
    latest: &str,
) -> Option<bool> {
    let latest = parse_version(latest)?;
    let requirement = nuget_requirement(requirement)?;
    crate::parse_semver_req(&requirement)
        .map(|requirement| requirement.matches(&latest))
        .ok()
}

pub(in crate::range) fn nuget_requirement(requirement: &str) -> Option<String> {
    let requirement = requirement.trim();
    let open = requirement.as_bytes().first().copied()?;
    let close = requirement.as_bytes().last().copied()?;
    if !matches!(open, b'[' | b'(') || !matches!(close, b']' | b')') {
        return None;
    }

    let inner = &requirement[1..requirement.len() - 1];
    let Some((min, max)) = inner.split_once(',') else {
        return normalize_nuget_bound(inner).map(|version| format!("={version}"));
    };

    let mut parts = vec![];
    if let Some(version) = normalize_nuget_bound(min) {
        parts.push(format!(
            "{}{version}",
            if open == b'[' { ">=" } else { ">" }
        ));
    }
    if let Some(version) = normalize_nuget_bound(max) {
        parts.push(format!(
            "{}{version}",
            if close == b']' { "<=" } else { "<" }
        ));
    }

    (!parts.is_empty()).then(|| parts.join(", "))
}

fn normalize_nuget_bound(bound: &str) -> Option<String> {
    Some(parse_padded_version(bound.trim())?.to_string())
}
