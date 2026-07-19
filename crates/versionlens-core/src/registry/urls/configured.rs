use versionlens_model::Dependency;
use versionlens_providers::{RegistryEndpoint, registry_endpoint_with_base};

use crate::RegistryUrlConfig;
use versionlens_model::Ecosystem::Maven;

pub(super) fn configured_registry_endpoints(
    registry_urls: &[RegistryUrlConfig],
    dependency: &Dependency,
) -> Vec<RegistryEndpoint> {
    registry_urls
        .iter()
        .filter(|url| url.ecosystem == dependency.ecosystem)
        .map(|url| {
            registry_endpoint_with_base(
                dependency.ecosystem,
                registry_package_name(dependency),
                Some(&url.url),
            )
        })
        .collect()
}

pub(super) fn extend_registry_endpoints(
    endpoints: &mut Vec<RegistryEndpoint>,
    registry_urls: &[RegistryUrlConfig],
    dependency: &Dependency,
) {
    endpoints.extend(configured_registry_endpoints(registry_urls, dependency));
}

pub(super) fn default_registry_endpoint(dependency: &Dependency) -> RegistryEndpoint {
    registry_endpoint_with_base(
        dependency.ecosystem,
        registry_package_name(dependency),
        None,
    )
}

pub(super) fn gradle_plugin_portal_registry_endpoint(
    dependency: &Dependency,
) -> Option<RegistryEndpoint> {
    if dependency.ecosystem == Maven
        && dependency.group == "plugins"
        && dependency.name.ends_with(".gradle.plugin")
    {
        Some(registry_endpoint_with_base(
            Maven,
            registry_package_name(dependency),
            Some("https://plugins.gradle.org/m2"),
        ))
    } else {
        None
    }
}

pub(super) fn clojars_registry_endpoint(dependency: &Dependency) -> RegistryEndpoint {
    registry_endpoint_with_base(
        Maven,
        registry_package_name(dependency),
        Some("https://repo.clojars.org"),
    )
}

fn registry_package_name(dependency: &Dependency) -> &str {
    dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name)
}
