use serde_json::Value;

pub(super) struct DockerTagEntry<'a> {
    pub(super) name: &'a str,
    pub(super) digest: Option<&'a str>,
}

pub(super) fn docker_tag_entries(value: &Value) -> Vec<DockerTagEntry<'_>> {
    let mut entries = object_tag_entries(value);
    entries.extend(registry_v2_tag_entries(value));
    entries
}

fn object_tag_entries(value: &Value) -> Vec<DockerTagEntry<'_>> {
    let docker_hub_response = value.get("results").is_some();
    value
        .get("results")
        .unwrap_or(value)
        .as_array()
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter(|entry| {
            !docker_hub_response
                || entry
                    .get("tag_status")
                    .and_then(Value::as_str)
                    .is_some_and(|status| status == "active")
        })
        .filter(|entry| {
            !docker_hub_response
                || entry
                    .get("digest")
                    .and_then(Value::as_str)
                    .is_some_and(|digest| !digest.is_empty())
        })
        .filter_map(|entry| {
            Some(DockerTagEntry {
                name: entry.get("name")?.as_str()?,
                digest: entry.get("digest").and_then(Value::as_str),
            })
        })
        .collect()
}

fn registry_v2_tag_entries(value: &Value) -> Vec<DockerTagEntry<'_>> {
    value
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flat_map(|tags| tags.iter())
        .filter_map(|tag| {
            Some(DockerTagEntry {
                name: tag.as_str()?,
                digest: None,
            })
        })
        .collect()
}
