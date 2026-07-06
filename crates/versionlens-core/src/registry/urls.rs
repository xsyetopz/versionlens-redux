use versionlens_parsers::{Dependency, Ecosystem};

use crate::VersionLensSession;
use crate::registry::RegistryContext;
use versionlens_parsers::Ecosystem::{Dotnet, Go, Maven, Python};
use versionlens_parsers::ManifestKind::{ClojureDepsEdn, LeiningenProjectClj};

mod configured;
mod dotnet;
mod hosted;

use configured::{
    clojars_registry_url, configured_registry_urls, default_registry_url, extend_registry_urls,
    gradle_plugin_portal_registry_url,
};
use hosted::hosted_registry_urls;

impl VersionLensSession {
    #[cfg(test)]
    pub(crate) fn registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.registry_urls_with_context(dependency, &crate::default())
    }

    pub(crate) fn registry_urls_with_context(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> Vec<String> {
        if let Some(urls) = hosted_registry_urls(dependency) {
            return urls;
        }

        if context.go_proxy_disabled_for_dependency(dependency) {
            return vec![];
        }

        if dependency.ecosystem == Dotnet {
            if context.has_dotnet_registry_configuration() {
                return context.dotnet_registry_urls(dependency);
            }

            let urls = self.dotnet_registry_urls();
            if !urls.is_empty() {
                return urls;
            }
        }

        let mut urls = configured_registry_urls(&self.config.providers.registry_urls, dependency);
        let is_clojure_maven_document = dependency.ecosystem == Maven
            && !context.maven_uses_mirror()
            && matches!(
                context.manifest_kind(),
                Some(ClojureDepsEdn | LeiningenProjectClj)
            );
        if is_clojure_maven_document {
            push_unique_url(&mut urls, default_registry_url(dependency));
            push_unique_url(&mut urls, clojars_registry_url(dependency));
        }
        if !python_dependency_has_named_source(dependency) {
            extend_registry_urls(&mut urls, &context.urls, dependency);
        }
        urls.extend(context.registry_urls(dependency));

        if dependency.ecosystem == Maven && !context.maven_uses_mirror() {
            if let Some(plugin_portal_url) = gradle_plugin_portal_registry_url(dependency)
                && !urls.iter().any(|url| url == &plugin_portal_url)
            {
                urls.push(plugin_portal_url);
            }
            if !is_clojure_maven_document {
                push_unique_url(&mut urls, default_registry_url(dependency));
            }
        }

        let urls = if urls.is_empty() && context.default_registry_disabled(dependency.ecosystem) {
            urls
        } else if urls.is_empty() {
            vec![default_registry_url(dependency)]
        } else {
            urls
        };

        go_module_proxy_urls_with_latest_fallback(dependency.ecosystem, urls)
    }
}

fn push_unique_url(urls: &mut Vec<String>, url: String) {
    if !urls.iter().any(|existing| existing == &url) {
        urls.push(url);
    }
}

fn go_module_proxy_urls_with_latest_fallback(
    ecosystem: Ecosystem,
    urls: Vec<String>,
) -> Vec<String> {
    if ecosystem != Go {
        return urls;
    }

    let mut expanded = vec![];
    for url in urls {
        if let Some(latest_url) = url
            .strip_suffix("/@v/list")
            .map(|base| format!("{base}/@latest"))
        {
            expanded.push(url);
            expanded.push(latest_url);
        } else {
            expanded.push(url);
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
