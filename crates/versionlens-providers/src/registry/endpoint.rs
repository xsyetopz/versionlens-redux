use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Go, Python};

use super::urls::{registry_url, registry_url_with_base};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryResponseKind {
    Ecosystem,
    GoModuleList,
    GoModuleLatest,
    PythonJson,
    PythonRss,
    PythonSimple,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEndpoint {
    pub url: String,
    pub response_kind: RegistryResponseKind,
}

impl RegistryEndpoint {
    pub fn new(url: String, response_kind: RegistryResponseKind) -> Self {
        Self { url, response_kind }
    }

    pub fn ecosystem(url: String) -> Self {
        Self::new(url, RegistryResponseKind::Ecosystem)
    }
}

pub fn registry_endpoint(ecosystem: Ecosystem, name: &str) -> RegistryEndpoint {
    RegistryEndpoint::new(
        registry_url(ecosystem, name),
        default_response_kind(ecosystem),
    )
}

pub fn registry_endpoint_with_base(
    ecosystem: Ecosystem,
    name: &str,
    base_url: Option<&str>,
) -> RegistryEndpoint {
    let response_kind = base_url
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map_or_else(
            || default_response_kind(ecosystem),
            |url| custom_response_kind(ecosystem, url),
        );
    RegistryEndpoint::new(
        registry_url_with_base(ecosystem, name, base_url),
        response_kind,
    )
}

fn default_response_kind(ecosystem: Ecosystem) -> RegistryResponseKind {
    match ecosystem {
        Go => RegistryResponseKind::GoModuleList,
        Python => RegistryResponseKind::PythonRss,
        _ => RegistryResponseKind::Ecosystem,
    }
}

fn custom_response_kind(ecosystem: Ecosystem, base_url: &str) -> RegistryResponseKind {
    match ecosystem {
        Go => RegistryResponseKind::GoModuleList,
        Python if is_python_simple_base(base_url) => RegistryResponseKind::PythonSimple,
        Python if base_url.trim_end_matches('/').ends_with("/json") => {
            RegistryResponseKind::PythonJson
        }
        _ => RegistryResponseKind::Ecosystem,
    }
}

pub(crate) fn is_python_simple_base(base_url: &str) -> bool {
    base_url
        .split(['?', '#'])
        .next()
        .unwrap_or(base_url)
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .is_some_and(|segment| segment.eq_ignore_ascii_case("simple"))
}
