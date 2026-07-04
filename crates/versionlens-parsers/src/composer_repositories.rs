use jsonc_parser::ast::{Object, Value};
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerAuthEntry {
    pub registry: String,
    pub header_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerRepository {
    pub url: String,
    pub only: Vec<String>,
    pub exclude: Vec<String>,
}

pub fn parse_composer_packagist_disabled(text: &str) -> bool {
    let Ok(parse_result) = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())
    else {
        return false;
    };
    let Some(Value::Object(root)) = parse_result.value else {
        return false;
    };

    match root.get("repositories").map(|property| &property.value) {
        Some(Value::Object(repositories)) => repositories.properties.iter().any(|property| {
            property.name.as_str() == "packagist.org" && value_is_false(&property.value)
        }),
        Some(Value::Array(repositories)) => repositories.elements.iter().any(packagist_false_entry),
        _ => false,
    }
}

pub fn parse_composer_repository_urls(text: &str) -> Vec<String> {
    parse_composer_repositories(text)
        .into_iter()
        .map(|repository| repository.url)
        .collect()
}

pub fn parse_composer_repositories(text: &str) -> Vec<ComposerRepository> {
    let Ok(parse_result) = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())
    else {
        return Vec::new();
    };
    let Some(Value::Object(root)) = parse_result.value else {
        return Vec::new();
    };

    match root.get("repositories").map(|property| &property.value) {
        Some(Value::Array(repositories)) => repositories
            .elements
            .iter()
            .filter_map(repository)
            .collect(),
        Some(Value::Object(repositories)) => repositories
            .properties
            .iter()
            .filter_map(|property| repository(&property.value))
            .collect(),
        _ => Vec::new(),
    }
}

pub fn parse_composer_auth_entries(text: &str) -> Vec<ComposerAuthEntry> {
    let Ok(parse_result) = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())
    else {
        return Vec::new();
    };
    let Some(Value::Object(root)) = parse_result.value else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    collect_http_basic_entries(&root, &mut entries);
    collect_bearer_entries(&root, &mut entries);
    entries
}

fn collect_http_basic_entries(root: &Object<'_>, out: &mut Vec<ComposerAuthEntry>) {
    let Some(registries) = root.get_object("http-basic") else {
        return;
    };

    for property in &registries.properties {
        let Value::Object(credentials) = &property.value else {
            continue;
        };
        let (Some(username), Some(password)) = (
            credentials.get_string("username"),
            credentials.get_string("password"),
        ) else {
            continue;
        };
        if let Some(registry) = normalized_registry(property.name.as_str()) {
            out.push(ComposerAuthEntry {
                registry,
                header_value: basic_header(username.value.as_ref(), password.value.as_ref()),
            });
        }
    }
}

fn collect_bearer_entries(root: &Object<'_>, out: &mut Vec<ComposerAuthEntry>) {
    let Some(registries) = root.get_object("bearer") else {
        return;
    };

    for property in &registries.properties {
        let Value::StringLit(token) = &property.value else {
            continue;
        };
        let token = token.value.as_ref().trim();
        if token.is_empty() {
            continue;
        }
        if let Some(registry) = normalized_registry(property.name.as_str()) {
            out.push(ComposerAuthEntry {
                registry,
                header_value: format!("Bearer {token}"),
            });
        }
    }
}

fn repository(value: &Value<'_>) -> Option<ComposerRepository> {
    let Value::Object(repository) = value else {
        return None;
    };
    composer_repository_url(repository).map(|url| ComposerRepository {
        url: url.to_owned(),
        only: string_array(repository, "only"),
        exclude: string_array(repository, "exclude"),
    })
}

fn packagist_false_entry(value: &Value<'_>) -> bool {
    let Value::Object(object) = value else {
        return false;
    };

    object
        .get("packagist.org")
        .is_some_and(|property| value_is_false(&property.value))
}

fn value_is_false(value: &Value<'_>) -> bool {
    matches!(value, Value::BooleanLit(boolean) if !boolean.value)
}

fn string_array(repository: &Object<'_>, field: &str) -> Vec<String> {
    let Some(Value::Array(array)) = repository.get(field).map(|property| &property.value) else {
        return Vec::new();
    };

    array
        .elements
        .iter()
        .filter_map(|element| match element {
            Value::StringLit(value) => Some(value.value.as_ref().to_owned()),
            _ => None,
        })
        .collect()
}

fn normalized_registry(registry: &str) -> Option<String> {
    let registry = registry
        .trim()
        .trim_end_matches('/')
        .strip_prefix("https://")
        .or_else(|| {
            registry
                .trim()
                .trim_end_matches('/')
                .strip_prefix("http://")
        })
        .unwrap_or_else(|| registry.trim().trim_end_matches('/'));
    (!registry.is_empty()).then(|| registry.to_owned())
}

fn basic_header(username: &str, password: &str) -> String {
    use base64::Engine;
    let credentials = format!("{username}:{password}");
    format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode(credentials)
    )
}

fn composer_repository_url<'a>(repository: &'a Object<'a>) -> Option<&'a str> {
    let repository_type = repository.get_string("type")?.value.as_ref();
    if repository_type != "composer" {
        return None;
    }

    repository.get_string("url").map(|url| url.value.as_ref())
}

#[cfg(test)]
mod tests;
