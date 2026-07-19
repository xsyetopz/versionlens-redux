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
    #[cfg(test)]
    pub(crate) fn registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.registry_urls_with_context(dependency, &crate::default())
    }

    #[cfg(test)]
    pub(crate) fn registry_urls_with_context(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> Vec<String> {
        self.registry_endpoints_with_context(dependency, context)
            .into_iter()
            .map(|endpoint| endpoint.url)
            .collect()
    }

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
mod tests;
