use versionlens_parsers::Dependency;
use versionlens_providers::registry_url_with_base;

use crate::RegistryUrlConfig;

pub(super) fn configured_registry_urls(
    registry_urls: &[RegistryUrlConfig],
    dependency: &Dependency,
) -> Vec<String> {
    registry_urls
        .iter()
        .filter(|url| url.ecosystem == dependency.ecosystem)
        .map(|url| {
            registry_url_with_base(
                dependency.ecosystem,
                registry_package_name(dependency),
                Some(&url.url),
            )
        })
        .collect()
}

pub(super) fn extend_registry_urls(
    urls: &mut Vec<String>,
    registry_urls: &[RegistryUrlConfig],
    dependency: &Dependency,
) {
    urls.extend(configured_registry_urls(registry_urls, dependency));
}

pub(super) fn default_registry_url(dependency: &Dependency) -> String {
    registry_url_with_base(
        dependency.ecosystem,
        registry_package_name(dependency),
        None,
    )
}

fn registry_package_name(dependency: &Dependency) -> &str {
    dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name)
}
