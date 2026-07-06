use semver::Version;
use std::cmp::Ordering::{
    Equal as OrderingEqual, Greater as OrderingGreater, Less as OrderingLess,
};
use versionlens_versions::requirement_satisfies_latest;

use crate::model::UpdateChoice;

type UpdateChoices = Vec<UpdateChoice>;

pub fn release_update_choices(
    requirement: &str,
    latest: &str,
    versions: &[String],
) -> UpdateChoices {
    release_update_choices_with_prereleases(requirement, latest, versions, true, &[])
}

pub fn release_update_choices_with_prereleases(
    requirement: &str,
    latest: &str,
    versions: &[String],
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> UpdateChoices {
    let mut choices = stable_update_choices(requirement, latest, versions);
    if include_prereleases {
        choices.extend(prerelease_update_choices(
            requirement,
            versions,
            prerelease_tags,
        ));
    }
    choices
}

fn stable_update_choices(requirement: &str, latest: &str, versions: &[String]) -> UpdateChoices {
    let stable_versions = stable_versions(versions);
    let Some(current) = crate::parse_semver(requirement.trim()).ok() else {
        return range_update_choices(requirement, latest, &stable_versions);
    };
    if stable_versions.is_empty() {
        return vec![];
    }

    let mut choices = vec![];
    if !current_matches_latest(&current, latest) {
        push_unique_choice(&mut choices, latest_choice_label(latest), latest, "update");
    }

    if let Some(version) = next_major(&current, versions, &stable_versions) {
        push_unique_choice(&mut choices, "major", &version, "updateMajor");
    }
    if let Some(version) = next_minor(&current, &stable_versions) {
        push_unique_choice(&mut choices, "minor", &version, "updateMinor");
    }
    if let Some(version) = next_patch(&current, &stable_versions) {
        push_unique_choice(&mut choices, "patch", &version, "updatePatch");
    }

    sort_choices_by_version_descending(&mut choices);
    choices
}

fn current_matches_latest(current: &Version, latest: &str) -> bool {
    crate::parse_semver(latest.trim())
        .ok()
        .is_some_and(|latest| {
            current.major == latest.major
                && current.minor == latest.minor
                && current.patch == latest.patch
                && current.pre == latest.pre
        })
}

fn range_update_choices(
    requirement: &str,
    latest: &str,
    stable_versions: &[(&str, Version)],
) -> UpdateChoices {
    if stable_versions.is_empty() || !looks_like_range_requirement(requirement) {
        return vec![];
    }

    let mut choices = vec![];
    if range_latest_update_is_useful(requirement, latest) {
        push_unique_choice(&mut choices, latest_choice_label(latest), latest, "update");
    }

    if let Some(version) = latest_satisfying_range(requirement, stable_versions) {
        if let Ok(current) = crate::parse_semver(&version) {
            if let Some(version) = next_range_major(&current, stable_versions) {
                push_unique_choice(&mut choices, "major", &version, "updateMajor");
            }
            if let Some(version) = next_minor(&current, stable_versions) {
                push_unique_choice(&mut choices, "minor", &version, "updateMinor");
            }
            if let Some(version) = next_patch(&current, stable_versions) {
                push_unique_choice(&mut choices, "patch", &version, "updatePatch");
            }
        }
        if !requirement_satisfies_latest(requirement, latest)
            && range_latest_update_is_useful(requirement, &version)
        {
            push_unique_choice(&mut choices, "bump", &version, "update");
        }
    }

    sort_choices_by_version_descending(&mut choices);
    choices
}

fn range_latest_update_is_useful(requirement: &str, latest: &str) -> bool {
    if !requirement_satisfies_latest(requirement, latest) {
        return true;
    }

    let Some(minimum) = minimum_version(requirement) else {
        return true;
    };
    let Some(latest) = crate::parse_semver(latest.trim()).ok() else {
        return true;
    };

    minimum.major != latest.major
        || minimum.minor != latest.minor
        || minimum.patch != latest.patch
        || minimum.pre != latest.pre
}

fn next_range_major(current: &Version, stable_versions: &[(&str, Version)]) -> Option<String> {
    let next_major = stable_versions
        .iter()
        .map(|(_, version)| version.major)
        .filter(|major| *major > current.major)
        .min()?;
    stable_versions
        .iter()
        .filter(|(_, version)| version.major == next_major)
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn latest_choice_label(latest: &str) -> &'static str {
    if crate::parse_semver(latest.trim())
        .ok()
        .is_some_and(|version| !version.pre.is_empty())
    {
        "latest prerelease"
    } else {
        "latest"
    }
}

fn latest_satisfying_range(
    requirement: &str,
    stable_versions: &[(&str, Version)],
) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(release, _)| requirement_satisfies_latest(requirement, release))
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn next_major(
    current: &Version,
    versions: &[String],
    stable_versions: &[(&str, Version)],
) -> Option<String> {
    let next_major = find_next_major(current, versions)?;

    stable_versions
        .iter()
        .filter(|(_, version)| version.major == next_major)
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn find_next_major(current: &Version, versions: &[String]) -> Option<u64> {
    let mut found_current = false;

    for raw in versions {
        let Ok(version) = crate::parse_semver(raw) else {
            if found_current {
                let major = loose_major(raw)?;
                if major > current.major {
                    return Some(major);
                }
            }
            continue;
        };
        if !version.pre.is_empty() {
            continue;
        }
        if !found_current {
            if version == *current {
                found_current = true;
            }
            continue;
        }
        if version.major > current.major {
            return Some(version.major);
        }
    }

    None
}

fn loose_major(raw: &str) -> Option<u64> {
    let major = raw.split_once('.')?.0;
    (!major.is_empty() && major.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| major.parse().ok())
        .flatten()
}

fn next_minor(current: &Version, stable_versions: &[(&str, Version)]) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(_, version)| version.major == current.major && version.minor > current.minor)
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn next_patch(current: &Version, stable_versions: &[(&str, Version)]) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(_, version)| {
            version.major == current.major
                && version.minor == current.minor
                && version.patch > current.patch
        })
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn stable_versions(versions: &[String]) -> Vec<(&str, Version)> {
    versions
        .iter()
        .filter_map(|version| stable_version(version.as_str()))
        .collect()
}

fn stable_version(release: &str) -> Option<(&str, Version)> {
    let version = crate::parse_semver(release).ok()?;
    version.pre.is_empty().then_some((release, version))
}

fn prerelease_update_choices(
    requirement: &str,
    versions: &[String],
    prerelease_tags: &[String],
) -> UpdateChoices {
    let Some(minimum) = minimum_version(requirement) else {
        return vec![];
    };

    let mut groups = vec![];
    for (order, raw) in versions.iter().enumerate() {
        if !prerelease_tag_allowed(raw, prerelease_tags) {
            continue;
        }
        let Some((key, label, version)) = prerelease_parts(raw) else {
            continue;
        };
        upsert_prerelease_group(
            &mut groups,
            PrereleaseGroup {
                key,
                label,
                raw: raw.to_owned(),
                version,
                order,
            },
        );
    }

    groups.sort_by_key(|group| group.order);
    let mut choices = groups
        .into_iter()
        .filter(|group| group.version > minimum)
        .map(|group| UpdateChoice {
            label: group.label,
            version: group.raw,
            command: "update".to_owned(),
        })
        .collect::<Vec<_>>();
    choices.reverse();
    choices
}

struct PrereleaseGroup {
    key: String,
    label: String,
    raw: String,
    version: Version,
    order: usize,
}

fn upsert_prerelease_group(groups: &mut Vec<PrereleaseGroup>, next: PrereleaseGroup) {
    if let Some(group) = groups.iter_mut().find(|group| group.key == next.key) {
        group.label = next.label;
        group.raw = next.raw;
        group.version = next.version;
        group.order = next.order;
        return;
    }

    groups.push(next);
}

fn prerelease_parts(raw: &str) -> Option<(String, String, Version)> {
    let version = crate::parse_semver(raw).ok()?;
    if version.pre.is_empty() {
        return None;
    }
    let suffix = prerelease_suffix(raw)?;
    let first = suffix.split('.').next()?;
    let key = friendly_prerelease_name(raw).unwrap_or_else(|| first.to_owned());
    let label = tag_label(first, suffix);
    Some((key, label, version))
}

fn prerelease_tag_allowed(raw: &str, allowed_tags: &[String]) -> bool {
    if allowed_tags.is_empty() {
        return true;
    }

    prerelease_suffix(raw)
        .and_then(|suffix| suffix.split('.').next())
        .is_some_and(|first| {
            allowed_tags
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(first))
        })
}

fn prerelease_suffix(raw: &str) -> Option<&str> {
    raw.split_once('-').map(|(_, suffix)| suffix)
}

fn tag_label(first: &str, suffix: &str) -> String {
    let label = first
        .chars()
        .take_while(|char| !char.is_ascii_digit() && *char != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if label.is_empty() {
        suffix.to_ascii_lowercase()
    } else {
        label
    }
}

fn friendly_prerelease_name(raw: &str) -> Option<String> {
    let lowered = raw.to_ascii_lowercase();
    for group in COMMON_PRERELEASE_IDENTITIES {
        for common in *group {
            if follows_hyphen(&lowered, common) {
                return Some((*common).to_owned());
            }
        }
    }
    None
}

fn follows_hyphen(value: &str, needle: &str) -> bool {
    value
        .match_indices(needle)
        .any(|(index, _)| index > 0 && value.as_bytes()[index - 1] == b'-')
}

const COMMON_PRERELEASE_IDENTITIES: &[&[&str]] = &[
    &["legacy"],
    &["alpha", "preview", "a"],
    &["beta", "b"],
    &["next"],
    &["milestone", "m"],
    &["rc", "cr"],
    &["snapshot"],
    &["release", "final", "ga"],
    &["sp"],
];

fn minimum_version(requirement: &str) -> Option<Version> {
    let token = minimum_version_token(requirement)?;
    let normalized = normalize_minimum_version_token(token)?;
    crate::parse_semver(&normalized).ok()
}

fn minimum_version_token(requirement: &str) -> Option<&str> {
    let trimmed = requirement.trim();
    if trimmed == "*" {
        return Some("0.0.0");
    }

    let first_range = trimmed.split("||").next()?.split(',').next()?.trim();
    let token = first_range
        .trim_start_matches(['^', '~', '>', '<', '=', 'v'])
        .split_whitespace()
        .next()?;
    (!token.is_empty()).then_some(token)
}

fn normalize_minimum_version_token(token: &str) -> Option<String> {
    let (core, prerelease) = token
        .split_once('-')
        .map_or((token, None), |(core, prerelease)| (core, Some(prerelease)));
    let parts = core.split('.').collect::<Vec<_>>();
    if parts.len() > 3 || parts.is_empty() {
        return None;
    }

    let mut normalized = vec![];
    for part in parts {
        normalized.push(normalize_version_part(part)?);
    }
    while normalized.len() < 3 {
        normalized.push("0".to_owned());
    }

    let version = normalized.join(".");
    match prerelease {
        Some(suffix) => Some(format!("{version}-{suffix}")),
        None => Some(version),
    }
}

fn normalize_version_part(part: &str) -> Option<String> {
    if part == "*" || part.eq_ignore_ascii_case("x") {
        return Some("0".to_owned());
    }
    part.chars()
        .all(|char| char.is_ascii_digit())
        .then(|| part.to_owned())
}

fn push_unique_choice(choices: &mut UpdateChoices, label: &str, version: &str, command: &str) {
    if choices.iter().any(|choice| choice.version == version) {
        return;
    }

    choices.push(UpdateChoice {
        label: label.to_owned(),
        version: version.to_owned(),
        command: command.to_owned(),
    });
}

fn sort_choices_by_version_descending(choices: &mut [UpdateChoice]) {
    choices.sort_by(|left, right| {
        match (
            crate::parse_semver(left.version.trim()),
            crate::parse_semver(right.version.trim()),
        ) {
            (Ok(left), Ok(right)) => right.cmp(&left),
            (Ok(_), Err(_)) => OrderingLess,
            (Err(_), Ok(_)) => OrderingGreater,
            (Err(_), Err(_)) => OrderingEqual,
        }
    });
}

fn looks_like_range_requirement(requirement: &str) -> bool {
    requirement.contains([
        '^', '~', '>', '<', '=', '*', '|', ',', '[', ']', '(', ')', ' ',
    ])
}

#[cfg(test)]
mod tests;
