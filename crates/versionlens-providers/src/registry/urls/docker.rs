use serde_json::Value;

use super::trim_end_slash;

const DOCKER_HUB_TAGS_PREFIX: &str = "https://hub.docker.com/v2/namespaces/";
const DOCKER_HUB_TAGS_SUFFIX: &str = "/tags";

pub(super) fn docker_registry_url(name: &str) -> String {
    if let Some(repository) = explicit_registry_repository(name) {
        return microsoft_docker_url(repository);
    }

    let (namespace, repo) = docker_namespace_repo(name);
    format!("https://hub.docker.com/v2/namespaces/{namespace}/repositories/{repo}/tags")
}

pub(super) fn docker_registry_url_with_base(base_url: &str, name: &str) -> String {
    let base_url = trim_end_slash(base_url);
    if base_url.is_empty() {
        return docker_registry_url(name);
    }

    let base_url = base_url.strip_suffix("/v2").unwrap_or(base_url);
    format!("{base_url}/v2/{name}/tags/list")
}

pub fn docker_hub_tags_page_url(url: &str, page: u8) -> Option<String> {
    is_docker_hub_tags_url(url)
        .then(|| format!("{url}?page={page}&page_size=100&ordering=last_updated"))
}

pub fn docker_hub_body_has_next_page(body: &str) -> bool {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| docker_hub_next_page_present(&value))
        .unwrap_or(false)
}

pub fn merge_docker_hub_response_pages(pages: Vec<String>) -> Option<String> {
    let mut pages = pages.into_iter();
    let first = pages.next()?;
    let Ok(mut merged) = serde_json::from_str::<Value>(&first) else {
        return Some(first);
    };

    for page in pages {
        append_docker_hub_results(&mut merged, &page);
    }

    Some(serde_json::to_string(&merged).unwrap_or(first))
}

fn docker_namespace_repo(name: &str) -> (&str, &str) {
    let mut parts = name.split('/');
    let Some(first) = parts.next() else {
        return ("library", name);
    };
    parts.next().map_or(("library", name), |repo| (first, repo))
}

fn explicit_registry_repository(name: &str) -> Option<&str> {
    let (registry, repository) = name.split_once('/')?;
    if !is_explicit_registry(registry) || repository.is_empty() {
        return None;
    }

    Some(repository)
}

fn is_explicit_registry(registry: &str) -> bool {
    registry == "localhost" || registry.contains('.') || registry.contains(':')
}

fn microsoft_docker_url(repository: &str) -> String {
    let (namespace, repo) = docker_namespace_repo(repository);
    format!("https://mcr.microsoft.com/api/v1/catalog/{namespace}/{repo}/tags?reg=mar")
}

fn is_docker_hub_tags_url(url: &str) -> bool {
    url.starts_with(DOCKER_HUB_TAGS_PREFIX) && url.ends_with(DOCKER_HUB_TAGS_SUFFIX)
}

fn docker_hub_next_page_present(value: &Value) -> Option<bool> {
    let next = value.get("next")?;
    Some(!next.is_null() && next.as_str().is_none_or(|url| !url.is_empty()))
}

fn append_docker_hub_results(merged: &mut Value, page: &str) {
    let Ok(mut page) = serde_json::from_str::<Value>(page) else {
        return;
    };
    let Some(page_results) = page.get_mut("results").and_then(Value::as_array_mut) else {
        return;
    };
    let Some(merged_results) = merged.get_mut("results").and_then(Value::as_array_mut) else {
        return;
    };

    merged_results.append(page_results);
}
