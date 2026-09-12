use super::UpdateChoices;
use serde_json::{Value, from_str};
use std::cmp::Ordering::Greater as OrderingGreater;
use versionlens_suggestions::{UpdateChoice, push_unique_choice};

pub(super) fn docker_update_choices(requirement: &str, latest: &str, body: &str) -> UpdateChoices {
    if latest.is_empty() || latest == requirement {
        return vec![];
    }

    let Some(current) = docker_tag_shape(requirement) else {
        return vec![UpdateChoice {
            label: "latest".to_owned(),
            version: latest.to_owned(),
            command: "update".to_owned(),
            replacement: None,
        }];
    };
    let updates = docker_matching_tag_shape_updates(&current, body);
    let latest_version = updates.last().map_or_else(
        || latest.to_owned(),
        |candidate| candidate.tag.as_str().to_owned(),
    );
    let mut choices = vec![];
    push_unique_choice(&mut choices, "latest", &latest_version, "update");

    if let Some(version) = docker_next_major_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "major", version, "updateMajor");
    }
    if let Some(version) = docker_next_minor_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "minor", version, "updateMinor");
    }
    if let Some(version) = docker_next_patch_update(&current.numbers, &updates) {
        push_unique_choice(&mut choices, "patch", version, "updatePatch");
    }

    choices
}

struct DockerTagShape {
    numbers: Vec<u64>,
    suffix: Option<String>,
}

struct DockerTagCandidate {
    tag: String,
    numbers: Vec<u64>,
}

fn docker_matching_tag_shape_updates(
    current: &DockerTagShape,
    body: &str,
) -> Vec<DockerTagCandidate> {
    let tags = docker_response_tag_names(body);
    let mut updates = vec![];

    for tag in tags {
        let Some(candidate) = docker_tag_shape(&tag) else {
            continue;
        };
        if candidate.suffix != current.suffix || candidate.numbers.len() != current.numbers.len() {
            continue;
        }
        if versionlens_versions::compare_numeric_segments(&candidate.numbers, &current.numbers)
            != OrderingGreater
        {
            continue;
        }
        updates.push(DockerTagCandidate {
            tag,
            numbers: candidate.numbers,
        });
    }

    updates.sort_by(|left, right| {
        versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
    });
    updates
}

fn docker_next_major_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() > current.first()
                && docker_trailing_components_are_zero(&candidate.numbers, 1)
        })
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
        .map(|candidate| candidate.tag.as_str())
}

fn docker_next_minor_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    let major = *current.first()?;
    let minor = *current.get(1)?;
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() == Some(&major)
                && candidate.numbers.get(1).is_some_and(|value| *value > minor)
                && docker_trailing_components_are_zero(&candidate.numbers, 2)
        })
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
        .map(|candidate| candidate.tag.as_str())
}

fn docker_next_patch_update<'a>(
    current: &[u64],
    updates: &'a [DockerTagCandidate],
) -> Option<&'a str> {
    let major = *current.first()?;
    let minor = *current.get(1)?;
    let patch = *current.get(2)?;
    updates
        .iter()
        .filter(|candidate| {
            candidate.numbers.first() == Some(&major)
                && candidate.numbers.get(1) == Some(&minor)
                && candidate.numbers.get(2).is_some_and(|value| *value > patch)
                && docker_trailing_components_are_zero(&candidate.numbers, 3)
        })
        .min_by(|left, right| {
            versionlens_versions::compare_numeric_segments(&left.numbers, &right.numbers)
        })
        .map(|candidate| candidate.tag.as_str())
}

fn docker_trailing_components_are_zero(numbers: &[u64], start: usize) -> bool {
    numbers.iter().skip(start).all(|value| *value == 0)
}

fn docker_tag_shape(tag: &str) -> Option<DockerTagShape> {
    let (version, suffix) = tag
        .split_once('-')
        .map_or((tag, None), |(version, suffix)| {
            (version, (!suffix.is_empty()).then_some(suffix))
        });
    let numbers = versionlens_versions::numeric_segments(version)?;
    Some(DockerTagShape {
        numbers,
        suffix: suffix.map(|value| value.to_owned()),
    })
}

fn docker_response_tag_names(body: &str) -> Vec<String> {
    let Ok(value) = from_str::<Value>(body) else {
        return vec![];
    };
    let mut tags = vec![];
    tags.extend(docker_object_tag_names(
        value.get("results").unwrap_or(&value),
    ));
    tags.extend(docker_registry_v2_tag_names(&value));
    tags
}

fn docker_object_tag_names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter_map(docker_object_tag_name)
        .map(|value| value.to_owned())
        .collect()
}

fn docker_object_tag_name(entry: &Value) -> Option<&str> {
    let status = entry.get("tag_status").and_then(|value| value.as_str());
    if status.is_some_and(|status| status != "active") {
        return None;
    }
    if status.is_some()
        && entry
            .get("digest")
            .and_then(|value| value.as_str())
            .is_none_or(str::is_empty)
    {
        return None;
    }
    entry.get("name")?.as_str()
}

fn docker_registry_v2_tag_names(value: &Value) -> Vec<String> {
    value
        .get("tags")
        .and_then(|value| value.as_array())
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter_map(|value| value.as_str())
        .map(|value| value.to_owned())
        .collect()
}
