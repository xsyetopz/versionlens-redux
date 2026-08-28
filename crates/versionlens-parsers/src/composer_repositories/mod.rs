use crate::support;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use jsonc_parser::ast::Value::BooleanLit as ComposerBooleanLit;
use jsonc_parser::ast::Value::{
    Array as JsonValueArray, Object as JsonValueObject, StringLit as JsonValueStringLit,
};
use jsonc_parser::ast::{Object, Value};

type ComposerJsonObject<'a> = Object<'a>;
type ComposerJsonValue<'a> = Value<'a>;
type ComposerAuthEntries = Vec<ComposerAuthEntry>;

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
    pub packages: Vec<ComposerRepositoryPackage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposerRepositoryPackage {
    pub name: String,
    pub version: String,
}

pub fn parse_composer_packagist_disabled(text: &str) -> bool {
    support::with_json_object(text, |root| {
        match root.get("repositories").map(|property| &property.value) {
            Some(JsonValueObject(repositories)) => repositories.properties.iter().any(|property| {
                property.name.as_str() == "packagist.org" && value_is_false(&property.value)
            }),
            Some(JsonValueArray(repositories)) => {
                repositories.elements.iter().any(packagist_false_entry)
            }
            _ => false,
        }
    })
    .unwrap_or(false)
}

pub fn parse_composer_repository_urls(text: &str) -> Vec<String> {
    parse_composer_repositories(text)
        .into_iter()
        .filter_map(|repository| (!repository.url.is_empty()).then_some(repository.url))
        .collect()
}

pub fn parse_composer_repositories(text: &str) -> Vec<ComposerRepository> {
    support::with_json_object(text, |root| {
        match root.get("repositories").map(|property| &property.value) {
            Some(JsonValueArray(repositories)) => repositories
                .elements
                .iter()
                .filter_map(repository)
                .collect(),
            Some(JsonValueObject(repositories)) => repositories
                .properties
                .iter()
                .filter_map(|property| repository(&property.value))
                .collect(),
            _ => vec![],
        }
    })
    .unwrap_or_default()
}

pub fn parse_composer_auth_entries(text: &str) -> ComposerAuthEntries {
    support::with_json_object(text, |root| {
        let mut entries = vec![];
        collect_http_basic_entries(root, &mut entries);
        collect_bearer_entries(root, &mut entries);
        entries
    })
    .unwrap_or_default()
}

fn collect_http_basic_entries(root: &ComposerJsonObject<'_>, out: &mut ComposerAuthEntries) {
    let Some(registries) = root.get_object("http-basic") else {
        return;
    };

    for property in &registries.properties {
        let JsonValueObject(credentials) = &property.value else {
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

fn collect_bearer_entries(root: &ComposerJsonObject<'_>, out: &mut ComposerAuthEntries) {
    let Some(registries) = root.get_object("bearer") else {
        return;
    };

    for property in &registries.properties {
        let JsonValueStringLit(token) = &property.value else {
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

fn repository(value: &ComposerJsonValue<'_>) -> Option<ComposerRepository> {
    let JsonValueObject(repository) = value else {
        return None;
    };
    let only = string_array(repository, "only");
    let exclude = string_array(repository, "exclude");
    if let Some(url) = composer_repository_url(repository) {
        return Some(ComposerRepository {
            url: url.to_owned(),
            only,
            exclude,
            packages: vec![],
        });
    }

    let packages = composer_repository_packages(repository);
    (!packages.is_empty()).then_some(ComposerRepository {
        url: "".to_owned(),
        only,
        exclude,
        packages,
    })
}

fn packagist_false_entry(value: &ComposerJsonValue<'_>) -> bool {
    let JsonValueObject(object) = value else {
        return false;
    };

    object
        .get("packagist.org")
        .is_some_and(|property| value_is_false(&property.value))
}

fn value_is_false(value: &ComposerJsonValue<'_>) -> bool {
    matches!(value, ComposerBooleanLit(boolean) if !boolean.value)
}

fn string_array(repository: &ComposerJsonObject<'_>, field: &str) -> Vec<String> {
    let Some(JsonValueArray(array)) = repository.get(field).map(|property| &property.value) else {
        return vec![];
    };

    array
        .elements
        .iter()
        .filter_map(|element| match element {
            JsonValueStringLit(value) => Some(value.value.as_ref().to_owned()),
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
    format!("Basic {}", BASE64_STANDARD.encode(credentials))
}

fn composer_repository_url<'a>(repository: &'a ComposerJsonObject<'a>) -> Option<&'a str> {
    let repository_type = repository.get_string("type")?.value.as_ref();
    if repository_type != "composer" {
        return None;
    }

    repository.get_string("url").map(|url| url.value.as_ref())
}

fn composer_repository_packages(
    repository: &ComposerJsonObject<'_>,
) -> Vec<ComposerRepositoryPackage> {
    let Some(repository_type) = repository.get_string("type") else {
        return vec![];
    };
    if repository_type.value.as_ref() != "package" {
        return vec![];
    }

    match repository.get("package").map(|property| &property.value) {
        Some(JsonValueObject(package)) => {
            composer_repository_package(package).into_iter().collect()
        }
        Some(JsonValueArray(packages)) => packages
            .elements
            .iter()
            .filter_map(|value| match value {
                JsonValueObject(package) => composer_repository_package(package),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

fn composer_repository_package(
    package: &ComposerJsonObject<'_>,
) -> Option<ComposerRepositoryPackage> {
    let name = package.get_string("name")?.value.as_ref().trim();
    let version = package.get_string("version")?.value.as_ref().trim();
    if name.is_empty() || version.is_empty() {
        return None;
    }

    Some(ComposerRepositoryPackage {
        name: name.to_owned(),
        version: version.to_owned(),
    })
}

#[cfg(test)]
mod tests;
