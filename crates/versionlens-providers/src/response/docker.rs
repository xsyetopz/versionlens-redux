use std::collections::BTreeSet;

use serde_json::Value;

mod entries;
mod latest;
mod version_key;

use entries::docker_tag_entries;

pub fn docker_tag_exists(body: &str, tag: &str) -> Option<bool> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    Some(
        docker_tag_entries(&value)
            .iter()
            .any(|entry| entry.name == tag),
    )
}

pub(crate) fn docker_build_versions(body: &str, requirement: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(body) else {
        return Vec::new();
    };
    let entries = docker_tag_entries(&value);
    let requirement = if requirement.is_empty() {
        "latest"
    } else {
        requirement
    };
    let Some(target_digest) = entries
        .iter()
        .find(|entry| entry.name == requirement)
        .and_then(|entry| entry.digest)
    else {
        return Vec::new();
    };

    let mut has_latest = false;
    let mut aliases = BTreeSet::new();
    let fixed_cores = docker_fixed_cores(&entries);
    for entry in entries
        .iter()
        .filter(|entry| entry.digest == Some(target_digest))
    {
        if entry.name == "latest" {
            has_latest = true;
        } else {
            aliases.insert(entry.name);
        }
    }
    if requirement != "latest"
        && let Some(core) = docker_canonical_core(requirement, &fixed_cores)
    {
        for entry in entries.iter().filter(|entry| {
            docker_canonical_core(entry.name, &fixed_cores)
                .is_some_and(|entry_core| entry_core == core)
        }) {
            aliases.insert(entry.name);
        }
    }
    if requirement == "latest"
        && let Some(core) = docker_latest_canonical_core(&entries, &fixed_cores, target_digest)
    {
        for entry in entries.iter().filter(|entry| {
            docker_canonical_core(entry.name, &fixed_cores)
                .is_some_and(|entry_core| entry_core == core)
        }) {
            aliases.insert(entry.name);
        }
    }

    let mut builds = Vec::with_capacity(aliases.len() + usize::from(has_latest));
    if has_latest {
        builds.push("latest".to_owned());
    }
    builds.extend(aliases.into_iter().map(str::to_owned));
    builds
}

fn docker_latest_canonical_core(
    entries: &[entries::DockerTagEntry<'_>],
    fixed_cores: &[[u64; 3]],
    target_digest: &str,
) -> Option<[u64; 3]> {
    entries
        .iter()
        .filter(|entry| entry.digest == Some(target_digest) && entry.name != "latest")
        .find_map(|entry| docker_canonical_core(entry.name, fixed_cores))
}

fn docker_fixed_cores(entries: &[entries::DockerTagEntry<'_>]) -> Vec<[u64; 3]> {
    entries
        .iter()
        .filter_map(|entry| {
            let (parts, len) = docker_numeric_parts(entry.name)?;
            (len == parts.len()).then_some(parts)
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn docker_canonical_core(tag: &str, fixed_cores: &[[u64; 3]]) -> Option<[u64; 3]> {
    let (parts, len) = docker_numeric_parts(tag)?;
    if len == parts.len() {
        return Some(parts);
    }
    fixed_cores
        .iter()
        .rev()
        .find(|core| core[..len] == parts[..len])
        .copied()
        .or(Some(parts))
}

fn docker_numeric_parts(tag: &str) -> Option<([u64; 3], usize)> {
    let version = tag.split_once('-').map_or(tag, |(version, _)| version);
    let mut parts = [0; 3];
    let mut len = 0;
    for part in version.split('.') {
        if len == parts.len() || part.is_empty() {
            return None;
        }
        parts[len] = part.parse::<u64>().ok()?;
        len += 1;
    }
    (len > 0).then_some((parts, len))
}

pub(crate) use latest::latest_docker_tag;
