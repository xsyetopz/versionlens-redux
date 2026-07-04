use toml_edit::{Document, Item, Value as TomlValue};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PoetrySource {
    pub(crate) name: String,
    pub(crate) url: String,
}

pub fn parse_python_registry_urls(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(parse_requirements_registry_url)
        .chain(parse_uv_pyproject_registry_urls(text))
        .chain(parse_poetry_source_urls(text))
        .collect()
}

pub fn parse_uv_registry_urls(text: &str) -> Vec<String> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };

    uv_registry_urls(document.as_item())
}

pub fn parse_pipfile_source_urls(text: &str) -> Vec<String> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(sources) = document.get("source").and_then(Item::as_array_of_tables) else {
        return Vec::new();
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

pub(crate) fn parse_poetry_sources(text: &str) -> Vec<PoetrySource> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(sources) = document
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("source"))
        .and_then(Item::as_array_of_tables)
    else {
        return Vec::new();
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
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(uv) = document.get("tool").and_then(|tool| tool.get("uv")) else {
        return Vec::new();
    };

    uv_registry_urls(uv)
}

fn uv_registry_urls(item: &Item) -> Vec<String> {
    let mut urls = Vec::new();
    push_uv_url(&mut urls, item.get("index-url"));
    push_uv_urls(&mut urls, item.get("extra-index-url"));
    push_uv_index_urls(&mut urls, item.get("index"));
    urls
}

fn push_uv_index_urls(urls: &mut Vec<String>, item: Option<&Item>) {
    let Some(array) = item.and_then(Item::as_array_of_tables) else {
        return;
    };

    urls.extend(
        array
            .iter()
            .filter_map(|table| table.get("url").and_then(toml_string_url)),
    );
}

fn push_uv_urls(urls: &mut Vec<String>, item: Option<&Item>) {
    if let Some(url) = item.and_then(toml_string_url) {
        urls.push(url);
        return;
    }

    if let Some(array) = item.and_then(Item::as_value).and_then(TomlValue::as_array) {
        urls.extend(array.iter().filter_map(uv_value_string));
    }
}

fn push_uv_url(urls: &mut Vec<String>, item: Option<&Item>) {
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
    let value = unquote(value).trim_end_matches('/');
    (!value.is_empty()).then(|| value.to_owned())
}

fn ignored_line(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#')
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

#[cfg(test)]
mod tests;
