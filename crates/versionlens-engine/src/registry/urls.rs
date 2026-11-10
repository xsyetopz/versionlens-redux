use versionlens_model::{Dependency, Ecosystem};
use versionlens_providers::{RegistryEndpoint, RegistryResponseKind};

use crate::VersionLensSession;
use crate::registry::{RegistryContext, RegistryEndpoints};
use versionlens_model::Ecosystem::{Dotnet, Go, Maven, Python};
use versionlens_model::ManifestKind::{ClojureDepsEdn, LeiningenProjectClj};

mod configured;
mod dotnet;
mod hosted;

use configured::{
    clojars_registry_endpoint, configured_registry_endpoints, default_registry_endpoint,
    extend_registry_endpoints, gradle_plugin_portal_registry_endpoint,
};
use hosted::hosted_registry_endpoints;

impl VersionLensSession {
    pub(crate) fn registry_endpoints_with_context(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> RegistryEndpoints {
        if let Some(endpoints) = hosted_registry_endpoints(dependency) {
            return endpoints;
        }

        if context.go_proxy_disabled_for_dependency(dependency) {
            return vec![];
        }

        if dependency.ecosystem == Dotnet {
            if context.has_dotnet_registry_configuration() {
                return context
                    .dotnet_registry_urls(dependency)
                    .into_iter()
                    .map(RegistryEndpoint::ecosystem)
                    .collect();
            }

            let urls = self.dotnet_registry_urls();
            if !urls.is_empty() {
                return urls.into_iter().map(RegistryEndpoint::ecosystem).collect();
            }
        }

        let mut endpoints =
            configured_registry_endpoints(&self.config.providers.registry_urls, dependency);
        let is_clojure_maven_document = dependency.ecosystem == Maven
            && !context.maven_uses_mirror()
            && matches!(
                context.manifest_kind(),
                Some(ClojureDepsEdn | LeiningenProjectClj)
            );
        if is_clojure_maven_document {
            push_unique_endpoint(&mut endpoints, default_registry_endpoint(dependency));
            push_unique_endpoint(&mut endpoints, clojars_registry_endpoint(dependency));
        }
        if !python_dependency_has_named_source(dependency) {
            extend_registry_endpoints(&mut endpoints, &context.urls, dependency);
        }
        endpoints.extend(context.registry_endpoints(dependency));

        if dependency.ecosystem == Maven && !context.maven_uses_mirror() {
            if let Some(plugin_portal) = gradle_plugin_portal_registry_endpoint(dependency)
                && !endpoints
                    .iter()
                    .any(|endpoint| endpoint.url == plugin_portal.url)
            {
                endpoints.push(plugin_portal);
            }
            if !is_clojure_maven_document {
                push_unique_endpoint(&mut endpoints, default_registry_endpoint(dependency));
            }
        }

        let endpoints =
            if endpoints.is_empty() && context.default_registry_disabled(dependency.ecosystem) {
                endpoints
            } else if endpoints.is_empty() {
                vec![default_registry_endpoint(dependency)]
            } else {
                endpoints
            };

        go_module_proxy_endpoints_with_latest_fallback(dependency.ecosystem, endpoints)
    }
}

fn push_unique_endpoint(endpoints: &mut Vec<RegistryEndpoint>, endpoint: RegistryEndpoint) {
    if !endpoints
        .iter()
        .any(|existing| existing.url == endpoint.url)
    {
        endpoints.push(endpoint);
    }
}

fn go_module_proxy_endpoints_with_latest_fallback(
    ecosystem: Ecosystem,
    endpoints: RegistryEndpoints,
) -> RegistryEndpoints {
    if ecosystem != Go {
        return endpoints;
    }

    let mut expanded = vec![];
    for endpoint in endpoints {
        if endpoint.response_kind == RegistryResponseKind::GoModuleList
            && let Some(latest_url) = endpoint
                .url
                .strip_suffix("/@v/list")
                .map(|base| format!("{base}/@latest"))
        {
            expanded.push(endpoint);
            expanded.push(RegistryEndpoint::new(
                latest_url,
                RegistryResponseKind::GoModuleLatest,
            ));
        } else {
            expanded.push(endpoint);
        }
    }
    expanded
}

fn python_dependency_has_named_source(dependency: &Dependency) -> bool {
    dependency.ecosystem == Python
        && dependency
            .hosted_url
            .as_deref()
            .is_some_and(|url| !url.contains("://"))
}

#[cfg(test)]
mod tests {
use crate::registry::RegistryContext;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;
use versionlens_model::DocumentInput;

use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
use versionlens_model::Ecosystem::*;

const GRADLE_PLUGIN_MARKER_URLS: &[&str] = &[
    "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
    "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
];

include!("tests/hosted.rs");
include!("tests/gradle/catalogs.rs");
include!("tests/gradle/scripts.rs");
include!("tests/jvm.rs");
include!("tests/configured.rs");
include!("tests/dotnet.rs");
fn package_file_fixture(name: &str) -> String {
    crate::test_support::fixture("tests/fixtures/core-scenarios/registry/urls/tests", name)
}

fn registry_input(uri: &str, language: &str, fixture: &str) -> DocumentInput {
    DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        package_file_fixture(fixture),
        None,
    )
}

fn registry_context_and_dependencies(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
) -> (
    crate::registry::RegistryContext,
    Vec<versionlens_model::Dependency>,
) {
    crate::test_support::registry_context_and_dependencies(session, input)
}

fn assert_first_registry_urls(
    session: &crate::VersionLensSession,
    input: &DocumentInput,
    expected: &[&str],
) {
    let (context, dependencies) = registry_context_and_dependencies(session, input);
    crate::test_support::assert_registry_urls(session, &context, &dependencies[0], expected);
}

fn assert_registry_urls_for_fixture(
    uri: &str,
    language: &str,
    fixture: &str,
    dependency_name: &str,
    expected_urls: &[&str],
) {
    let session = crate::test_support::test_session(false);
    let input = DocumentInput::new(
        uri.to_owned(),
        language.to_owned(),
        package_file_fixture(fixture),
        None,
    );
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(dependencies[0].name, dependency_name);
    crate::test_support::assert_registry_urls(&session, &context, &dependencies[0], expected_urls);
}

struct WorkspaceRegistryCase<'a> {
    root_name: &'a str,
    settings: &'a str,
    relative_document: &'a str,
    language: &'a str,
    fixture: &'a str,
    dependency_name: &'a str,
    expected_urls: &'a [&'a str],
}

fn assert_workspace_registry_urls(case: WorkspaceRegistryCase<'_>) {
    let root = temp_dir().join(format!("versionlens-{}-{}", case.root_name, id()));
    let document = root.join(case.relative_document);
    if let Some(parent) = document.parent() {
        create_dir_all(parent).unwrap();
    }
    write(root.join("settings.gradle"), case.settings).unwrap();

    let session = crate::test_support::test_session(false);
    let input = DocumentInput::new(
        format!("file://{}", document.display()),
        case.language.to_owned(),
        package_file_fixture(case.fixture),
        Some(root.to_string_lossy().into_owned()),
    );
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(dependencies[0].name, case.dependency_name);
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        case.expected_urls
            .iter()
            .map(|url| (*url).to_owned())
            .collect::<Vec<_>>()
    );

    remove_dir_all(root).unwrap();
}

impl crate::VersionLensSession {
    pub(crate) fn registry_urls(&self, dependency: &versionlens_model::Dependency) -> Vec<String> {
        self.registry_urls_with_context(dependency, &RegistryContext::default())
    }

    pub(crate) fn registry_urls_with_context(
        &self,
        dependency: &versionlens_model::Dependency,
        context: &crate::registry::RegistryContext,
    ) -> Vec<String> {
        self.registry_endpoints_with_context(dependency, context)
            .into_iter()
            .map(|endpoint| endpoint.url)
            .collect()
    }
}
}
