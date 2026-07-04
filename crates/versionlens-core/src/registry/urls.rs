use versionlens_parsers::{Dependency, Ecosystem};

use crate::VersionLensSession;
use crate::registry::RegistryContext;

mod configured;
mod dotnet;
mod hosted;

use configured::{configured_registry_urls, default_registry_url, extend_registry_urls};
use hosted::hosted_registry_urls;

impl VersionLensSession {
    #[cfg(test)]
    pub(crate) fn registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.registry_urls_with_context(dependency, &RegistryContext::default())
    }

    pub(crate) fn registry_urls_with_context(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> Vec<String> {
        if let Some(urls) = hosted_registry_urls(dependency) {
            return urls;
        }

        if dependency.ecosystem == Ecosystem::Dotnet {
            if context.has_dotnet_registry_configuration() {
                return context.dotnet_registry_urls(dependency);
            }

            let urls = self.dotnet_registry_urls(dependency);
            if !urls.is_empty() {
                return urls;
            }
        }

        let mut urls = configured_registry_urls(&self.config.providers.registry_urls, dependency);
        extend_registry_urls(&mut urls, &context.urls, dependency);
        urls.extend(context.registry_urls(dependency));

        if dependency.ecosystem == Ecosystem::Maven && !context.maven_uses_mirror() {
            let default_url = default_registry_url(dependency);
            if !urls.iter().any(|url| url == &default_url) {
                urls.push(default_url);
            }
        }

        if urls.is_empty() && context.default_registry_disabled(dependency.ecosystem) {
            urls
        } else if urls.is_empty() {
            vec![default_registry_url(dependency)]
        } else {
            urls
        }
    }
}

#[cfg(test)]
mod tests;
