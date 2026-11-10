use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Go, Python};

use super::urls::{registry_url, registry_url_with_base};

/// The format of a registry response body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryResponseKind {
    /// The ecosystem's own response format.
    Ecosystem,
    /// A Go module version list.
    GoModuleList,
    /// A Go module `@latest` document.
    GoModuleLatest,
    /// A Python JSON API document.
    PythonJson,
    /// A Python RSS feed.
    PythonRss,
    /// A Python simple index page.
    PythonSimple,
}

/// A registry URL and the format of the response it returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEndpoint {
    /// The URL to request.
    pub url: String,
    /// The format of the response body.
    pub response_kind: RegistryResponseKind,
}

impl RegistryEndpoint {
    /// Creates an endpoint from a URL and its response format.
    pub fn new(url: String, response_kind: RegistryResponseKind) -> Self {
        Self { url, response_kind }
    }

    /// Creates an endpoint that answers in the ecosystem's own format.
    pub fn ecosystem(url: String) -> Self {
        Self::new(url, RegistryResponseKind::Ecosystem)
    }
}

/// The default registry endpoint for a package.
pub fn registry_endpoint(ecosystem: Ecosystem, name: &str) -> RegistryEndpoint {
    RegistryEndpoint::new(
        registry_url(ecosystem, name),
        default_response_kind(ecosystem),
    )
}

/// The registry endpoint for a package, using a custom base URL when one is given.
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
