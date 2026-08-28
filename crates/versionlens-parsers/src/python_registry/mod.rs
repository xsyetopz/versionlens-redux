use toml_edit::{Array as TomlArray, Item, Value as TomlValue};

type UvRegistryUrls = Vec<String>;
type UvRegistryItem<'a> = Option<&'a Item>;

#[derive(Debug, PartialEq, Eq)]
pub struct PoetrySource {
    pub name: String,
    pub url: String,
}

pub fn parse_python_registry_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(parse_requirements_registry_url)
        .chain(parse_uv_pyproject_registry_urls(text))
        .chain(parse_poetry_source_urls(text))
        .collect()
}

pub fn parse_uv_registry_urls(text: &str) -> UvRegistryUrls {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };

    uv_registry_urls(document.as_item())
}

pub fn parse_pipfile_source_urls(text: &str) -> Vec<String> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let Some(sources) = document
        .get("source")
        .and_then(|value| value.as_array_of_tables())
    else {
        return vec![];
    };

    sources
        .iter()
        .filter_map(|table| table.get("url").and_then(toml_string_url))
        .collect()
}

pub fn parse_poetry_source_urls(text: &str) -> Vec<String> {
    parse_poetry_sources(text)
        .into_iter()
        .map(|source| source.url)
        .collect()
}

pub fn parse_poetry_sources(text: &str) -> Vec<PoetrySource> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let Some(sources) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("source"))
        .and_then(|value| value.as_array_of_tables())
    else {
        return vec![];
    };

    sources
        .iter()
        .filter_map(|table| {
            let name = table.get("name").and_then(toml_string_url)?;
            let url = table.get("url").and_then(toml_string_url)?;
            Some(PoetrySource { name, url })
        })
        .collect()
}

pub fn parse_pip_conf_registry_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(parse_pip_conf_registry_url)
        .collect()
}

pub fn parse_pip_env_registry_urls(env: &[(String, String)]) -> Vec<String> {
    env.iter()
        .filter_map(|(key, value)| pip_env_registry_values(key, value))
        .flatten()
        .collect()
}

fn parse_requirements_registry_url(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return None;
    }

    parse_option_value(trimmed, "--index-url")
        .or_else(|| parse_option_value(trimmed, "-i"))
        .or_else(|| parse_option_value(trimmed, "--extra-index-url"))
}

fn pip_env_registry_values(key: &str, value: &str) -> Option<Vec<String>> {
    match key {
        "PIP_INDEX_URL" => clean_url_value(value).map(|url| vec![url]),
        "PIP_EXTRA_INDEX_URL" => Some(
            value
                .split_whitespace()
                .filter_map(clean_url_value)
                .collect(),
        ),
        _ => None,
    }
}

fn parse_uv_pyproject_registry_urls(text: &str) -> Vec<String> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let Some(uv) = document.get("tool").and_then(|tool| tool.get("uv")) else {
        return vec![];
    };

    uv_registry_urls(uv)
}

fn uv_registry_urls(item: &Item) -> UvRegistryUrls {
    let mut urls = vec![];
    push_uv_url(&mut urls, item.get("index-url"));
    push_uv_urls(&mut urls, item.get("extra-index-url"));
    push_uv_index_urls(&mut urls, item.get("index"));
    urls
}

fn push_uv_index_urls(urls: &mut UvRegistryUrls, item: UvRegistryItem<'_>) {
    let Some(array) = item.and_then(|value| value.as_array_of_tables()) else {
        return;
    };

    urls.extend(
        array
            .iter()
            .filter_map(|table| table.get("url").and_then(toml_string_url)),
    );
}

fn toml_item_value(item: &Item) -> Option<&TomlValue> {
    item.as_value()
}

fn toml_value_array(value: &TomlValue) -> Option<&TomlArray> {
    value.as_array()
}

fn push_uv_urls(urls: &mut UvRegistryUrls, item: UvRegistryItem<'_>) {
    if let Some(url) = item.and_then(toml_string_url) {
        urls.push(url);
        return;
    }

    if let Some(array) = item.and_then(toml_item_value).and_then(toml_value_array) {
        urls.extend(array.iter().filter_map(uv_value_string));
    }
}

fn push_uv_url(urls: &mut UvRegistryUrls, item: UvRegistryItem<'_>) {
    if let Some(url) = item.and_then(toml_string_url) {
        urls.push(url);
    }
}

fn toml_string_url(item: &Item) -> Option<String> {
    item.as_value().and_then(uv_value_string)
}

fn uv_value_string(value: &TomlValue) -> Option<String> {
    value.as_str().and_then(clean_url_value)
}

fn parse_pip_conf_registry_url(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if ignored_line(trimmed) || trimmed.starts_with('[') {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    matches!(key, "index-url" | "extra-index-url")
        .then(|| clean_url_value(value))
        .flatten()
}

fn parse_option_value(line: &str, option: &str) -> Option<String> {
    if line == option {
        return None;
    }

    line.strip_prefix(&format!("{option}="))
        .or_else(|| line.strip_prefix(&format!("{option} ")))
        .and_then(clean_url_value)
}

fn clean_url_value(value: &str) -> Option<String> {
    let value = value.split('#').next().unwrap_or_default().trim();
    let value = versionlens_model::strip_matching_quotes(value).trim_end_matches('/');
    (!value.is_empty()).then(|| value.to_owned())
}

fn ignored_line(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#')
}

#[cfg(test)]
mod tests;
