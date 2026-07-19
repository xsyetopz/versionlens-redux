use std::path::PathBuf;
use versionlens_model::Ecosystem::{Cargo, Composer, Dotnet, Go, Hex, Maven, Npm, Python, Ruby};
use versionlens_model::ManifestKind::{
    CargoToml, ClojureDepsEdn, ComposerJson, DenoImportMapJson, DenoJson, DotnetProjectJson,
    DotnetXml, Gemfile, GleamToml, GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml,
    JsrJson, LeiningenProjectClj, MavenPomXml, MixExs, NpmPackageJson, NpmPackageYaml,
    PaketDependencies, PaketReferences, PnpmYaml, PythonPipfile, PythonPyprojectToml,
    PythonRequirementsTxt, RebarConfig, RubyGemspec, SbtBuild,
};
#[cfg(test)]
use versionlens_parsers::classify_document;

use versionlens_http::{HttpConfig, HttpHeader};
use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind};
use versionlens_parsers::{
    CargoRegistrySource, ComposerAuthEntry, ComposerRepository, MavenAuthEntry, MavenMirror,
    MavenNamedRepository, PoetrySource, parse_cargo_config_registry_sources,
    parse_clojure_maven_repositories, parse_composer_auth_entries,
    parse_composer_packagist_disabled, parse_composer_repositories, parse_gemfile_source_urls,
    parse_go_proxy_urls, parse_gradle_dependency_maven_repositories,
    parse_gradle_maven_repositories, parse_gradle_plugin_maven_repositories,
    parse_leiningen_maven_repositories, parse_maven_pom_repositories,
    parse_maven_settings_auth_entries, parse_maven_settings_mirror_urls,
    parse_maven_settings_mirrors, parse_maven_settings_repositories, parse_pip_conf_registry_urls,
    parse_pip_env_registry_urls, parse_pipfile_source_urls, parse_poetry_sources,
    parse_python_registry_urls, parse_sbt_maven_repositories, parse_uv_registry_urls,
};
use versionlens_providers::registry_endpoint_with_base;

use crate::RegistryUrlConfig;

use super::RegistryEndpoints;

mod dotnet;
mod npm;

use dotnet::{DotnetContext, dotnet_context};
use npm::{
    NpmContext, best_npm_auth_entry, best_npm_client_cert_entry, npm_context,
    npm_generic_proxy_for_request, npm_no_proxy_matches, npm_registry_entry_applies,
};

#[derive(Debug, Default)]
pub(crate) struct RegistryContext {
    manifest_kind: Option<ManifestKind>,
    pub(crate) urls: Vec<RegistryUrlConfig>,
    composer: ComposerContext,
    npm: NpmContext,
    dotnet: DotnetContext,
    maven: MavenContext,
    cargo_registries: Vec<CargoRegistrySource>,
    python_sources: Vec<PoetrySource>,
    go: GoContext,
}

#[derive(Debug, Default)]
struct ComposerContext {
    auth_entries: Vec<ComposerAuthEntry>,
    repositories: Vec<ComposerRepository>,
    packagist_disabled: bool,
}

#[derive(Debug, Default)]
struct MavenContext {
    plugin_urls: Vec<RegistryUrlConfig>,
    auth_entries: Vec<MavenAuthEntry>,
    uses_mirror: bool,
}

#[derive(Debug, Default)]
struct GoContext {
    proxy_disables_default: bool,
    no_proxy_patterns: Vec<String>,
}

#[cfg(test)]
pub(crate) fn registry_context_from_document(input: &DocumentInput) -> RegistryContext {
    <RegistryContext>::from_document(input)
}

pub(crate) fn registry_context_from_document_kind(
    input: &DocumentInput,
    kind: ManifestKind,
) -> RegistryContext {
    <RegistryContext>::from_document_kind(input, kind)
}

impl RegistryContext {
    #[cfg(test)]
    pub(crate) fn from_document(input: &DocumentInput) -> Self {
        let kind = classify_document(input);
        Self::from_document_kind(input, kind)
    }

    pub(crate) fn from_document_kind(input: &DocumentInput, kind: ManifestKind) -> Self {
        let mut context = match kind {
            CargoToml => Self::from_cargo_document(input),
            ComposerJson => Self::from_composer_document(input),
            DotnetProjectJson | DotnetXml | PaketDependencies | PaketReferences => {
                Self::from_dotnet_document(input)
            }
            Gemfile | RubyGemspec => Self::from_ruby_document(input),
            GoMod => Self::from_go_document(input),
            MavenPomXml
            | GradleBuild
            | GradleSettings
            | GradleVersionCatalogToml
            | SbtBuild
            | ClojureDepsEdn
            | LeiningenProjectClj => Self::from_maven_document(input, kind),
            GleamToml | MixExs | RebarConfig => Self::from_hex_document(input, kind),
            DenoJson | DenoImportMapJson | JsrJson | NpmPackageJson | NpmPackageYaml | PnpmYaml => {
                Self::from_npm_document(input)
            }
            PythonPipfile | PythonPyprojectToml | PythonRequirementsTxt => {
                Self::from_python_document(input)
            }
            _ => Self::default(),
        };
        context.manifest_kind = Some(kind);
        context
    }

    pub(crate) fn manifest_kind(&self) -> Option<ManifestKind> {
        self.manifest_kind
    }

    fn from_composer_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: vec![],
            composer: composer_context(input),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }

    fn from_cargo_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: vec![],
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: cargo_config_texts(input)
                .iter()
                .flat_map(|text| parse_cargo_config_registry_sources(text))
                .collect(),
            python_sources: vec![],
            go: crate::default(),
        }
    }

    fn from_dotnet_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: vec![],
            composer: crate::default(),
            npm: crate::default(),
            dotnet: dotnet_context(input),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }

    fn from_ruby_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: parse_gemfile_source_urls(&input.text)
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Ruby,
                    url,
                })
                .collect(),
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }

    fn from_go_document(input: &DocumentInput) -> Self {
        let env = env_entries(input);
        Self {
            manifest_kind: None,
            urls: parse_go_proxy_urls(&env)
                .into_iter()
                .map(|url| RegistryUrlConfig { ecosystem: Go, url })
                .collect(),
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: GoContext {
                proxy_disables_default: go_proxy_disables_default_registry(&env),
                no_proxy_patterns: go_no_proxy_patterns(&env),
            },
        }
    }

    fn from_hex_document(input: &DocumentInput, kind: ManifestKind) -> Self {
        Self {
            manifest_kind: None,
            urls: hex_registry_url_configs(input, kind)
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Hex,
                    url,
                })
                .collect(),
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }

    fn from_maven_document(input: &DocumentInput, kind: ManifestKind) -> Self {
        Self {
            manifest_kind: None,
            urls: parse_maven_registry_urls(input, kind)
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Maven,
                    url,
                })
                .collect(),
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: MavenContext {
                plugin_urls: parse_gradle_plugin_registry_urls(input, kind),
                auth_entries: maven_auth_entries(input),
                uses_mirror: maven_uses_mirror(input),
            },
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }

    pub(crate) fn has_urls(&self) -> bool {
        !self.urls.is_empty()
            || !self.composer.auth_entries.is_empty()
            || !self.composer.repositories.is_empty()
            || !self.npm.registries.is_empty()
            || !self.npm.auth_entries.is_empty()
            || !self.npm.client_cert_entries.is_empty()
            || self.npm.http.strict_ssl.is_some()
            || self.npm.http.proxy.is_some()
            || self.npm.http.no_proxy.is_some()
            || self.npm.http.proxy_disabled
            || self.npm.http.ca_file.is_some()
            || self.npm.http.ca.is_some()
            || self.npm.http.cert.is_some()
            || self.npm.http.key.is_some()
            || self.dotnet.has_urls()
            || !self.maven.auth_entries.is_empty()
            || self.maven.uses_mirror
            || !self.cargo_registries.is_empty()
            || !self.maven.plugin_urls.is_empty()
    }

    pub(crate) fn maven_uses_mirror(&self) -> bool {
        self.maven.uses_mirror
    }

    pub(crate) fn go_proxy_disabled_for_dependency(&self, dependency: &Dependency) -> bool {
        dependency.ecosystem == Go
            && (self.go.proxy_disables_default
                || go_module_matches_patterns(&dependency.name, &self.go.no_proxy_patterns))
    }

    pub(crate) fn default_registry_disabled(&self, ecosystem: Ecosystem) -> bool {
        (ecosystem == Composer && self.composer.packagist_disabled)
            || (ecosystem == Go && self.go.proxy_disables_default)
    }

    pub(crate) fn has_dotnet_registry_configuration(&self) -> bool {
        self.dotnet.has_registry_configuration()
    }

    pub(crate) fn dotnet_registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.dotnet.registry_urls(dependency)
    }

    pub(crate) fn registry_endpoints(&self, dependency: &Dependency) -> RegistryEndpoints {
        if dependency.ecosystem == Cargo {
            return self.cargo_registry_endpoints(dependency);
        }

        if dependency.ecosystem == Composer {
            return self.composer_registry_endpoints(dependency);
        }

        if dependency.ecosystem == Python {
            return self.python_registry_endpoints(dependency);
        }

        if dependency.ecosystem == Maven && is_gradle_plugin_marker(dependency) {
            return self.gradle_plugin_registry_endpoints(dependency);
        }

        if dependency.ecosystem != Npm {
            return vec![];
        }

        if let Some(url) = self
            .npm
            .registries
            .iter()
            .find(|entry| {
                entry.scope.is_some() && npm_registry_entry_applies(entry, &dependency.name)
            })
            .map(|entry| registry_endpoint_with_base(Npm, &dependency.name, Some(&entry.url)))
        {
            return vec![url];
        }

        self.npm
            .registries
            .iter()
            .find(|entry| entry.scope.is_none())
            .map(|entry| registry_endpoint_with_base(Npm, &dependency.name, Some(&entry.url)))
            .into_iter()
            .collect()
    }

    fn python_registry_endpoints(&self, dependency: &Dependency) -> RegistryEndpoints {
        let Some(source_name) = dependency.hosted_url.as_deref() else {
            return vec![];
        };
        if source_name.contains("://") {
            return vec![];
        }

        self.python_sources
            .iter()
            .find(|source| source.name == source_name)
            .map(|source| registry_endpoint_with_base(Python, &dependency.name, Some(&source.url)))
            .into_iter()
            .collect()
    }

    fn composer_registry_endpoints(&self, dependency: &Dependency) -> RegistryEndpoints {
        self.composer
            .repositories
            .iter()
            .filter(|repository| composer_repository_applies(repository, &dependency.name))
            .filter(|repository| !repository.url.is_empty())
            .map(|repository| {
                registry_endpoint_with_base(Composer, &dependency.name, Some(&repository.url))
            })
            .collect()
    }

    fn gradle_plugin_registry_endpoints(&self, dependency: &Dependency) -> RegistryEndpoints {
        self.maven
            .plugin_urls
            .iter()
            .map(|config| {
                registry_endpoint_with_base(
                    Maven,
                    dependency
                        .hosted_name
                        .as_deref()
                        .unwrap_or(&dependency.name),
                    Some(&config.url),
                )
            })
            .collect()
    }

    pub(crate) fn composer_inline_package_version(
        &self,
        dependency: &Dependency,
    ) -> Option<String> {
        if dependency.ecosystem != Composer {
            return None;
        }

        self.composer
            .repositories
            .iter()
            .filter(|repository| composer_repository_applies(repository, &dependency.name))
            .flat_map(|repository| &repository.packages)
            .find(|package| package.name == dependency.name)
            .map(|package| package.version.as_str().to_owned())
    }

    fn cargo_registry_endpoints(&self, dependency: &Dependency) -> RegistryEndpoints {
        let registry_name = dependency.hosted_url.as_deref().unwrap_or("crates-io");
        let Some(url) = cargo_registry_source_url(&self.cargo_registries, registry_name) else {
            return vec![];
        };

        vec![registry_endpoint_with_base(
            Cargo,
            dependency
                .hosted_name
                .as_deref()
                .unwrap_or(&dependency.name),
            Some(url),
        )]
    }

    pub(crate) fn auth_headers_for_url(&self, ecosystem: Ecosystem, url: &str) -> Vec<HttpHeader> {
        match ecosystem {
            Composer => auth_header(best_composer_auth_entry(&self.composer.auth_entries, url)),
            Dotnet => self.dotnet.auth_headers_for_url(url),
            Maven => auth_header(best_maven_auth_entry(&self.maven.auth_entries, url)),
            Npm => auth_header(best_npm_auth_entry(&self.npm.auth_entries, url)),
            _ => vec![],
        }
    }

    pub(crate) fn http_config_for_request(
        &self,
        ecosystem: Ecosystem,
        url: &str,
        base: HttpConfig,
    ) -> HttpConfig {
        if ecosystem != Npm {
            return base;
        }

        let is_npm_registry_fetch =
            !Self::starts_with_ignore_ascii_case(url, "https://api.github.com/repos/");
        let client_cert = best_npm_client_cert_entry(&self.npm.client_cert_entries, url);
        HttpConfig {
            timeout_ms: if is_npm_registry_fetch {
                npm_registry_timeout_ms(self.npm.http.timeout_ms)
            } else {
                base.timeout_ms
            },
            strict_ssl: self.npm.http.strict_ssl.unwrap_or(base.strict_ssl),
            proxy: self.proxy_for_request(url, base.proxy),
            ca_file: self
                .npm
                .http
                .ca_file
                .as_deref()
                .map(|value| value.to_owned())
                .or(base.ca_file),
            ca: self
                .npm
                .http
                .ca
                .as_deref()
                .map(|value| value.to_owned())
                .or(base.ca),
            cert_file: client_cert
                .and_then(|entry| entry.cert_file.as_deref())
                .map(|value| value.to_owned())
                .or(base.cert_file),
            key_file: client_cert
                .and_then(|entry| entry.key_file.as_deref())
                .map(|value| value.to_owned())
                .or(base.key_file),
            cert: client_cert
                .is_none()
                .then(|| self.npm.http.cert.as_deref())
                .flatten()
                .map(|value| value.to_owned())
                .or(base.cert),
            key: client_cert
                .is_none()
                .then(|| self.npm.http.key.as_deref())
                .flatten()
                .map(|value| value.to_owned())
                .or(base.key),
            auth_headers: base.auth_headers,
        }
    }

    fn proxy_for_request(&self, url: &str, base_proxy: Option<String>) -> Option<String> {
        if self.npm.http.proxy_disabled
            || npm_no_proxy_matches(url, self.npm.http.no_proxy.as_deref())
        {
            return None;
        }

        self.npm
            .http
            .proxy
            .as_deref()
            .map(|value| value.to_owned())
            .or_else(|| npm_generic_proxy_for_request(url, &self.npm.http.generic_proxy))
            .or(base_proxy)
    }

    fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
        value
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
    }

    fn from_python_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: python_registry_url_configs(input),
            composer: crate::default(),
            npm: crate::default(),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: parse_poetry_sources(&input.text),
            go: crate::default(),
        }
    }

    fn from_npm_document(input: &DocumentInput) -> Self {
        let process_env = process_env_entries();
        let project_npmrc_path = selected_dot_file_path(input, ".npmrc");
        let project_yarnrc_path = selected_project_yarnrc_path(input);
        let project_bunfig_path = selected_project_bunfig_path(input);
        let env = npm_env_entries(
            input,
            project_npmrc_path
                .as_ref()
                .or(project_yarnrc_path.as_ref())
                .or(project_bunfig_path.as_ref()),
            &process_env,
        );
        let npmrc_texts = npmrc_texts(input, project_npmrc_path, &process_env);
        let yarnrc_texts =
            dot_texts_or_candidates(input, project_yarnrc_path, &[".yarnrc.yml", ".yarnrc.yaml"]);
        let bunfig_texts =
            dot_texts_or_candidates(input, project_bunfig_path, &["bunfig.toml", ".bunfig.toml"]);
        Self {
            manifest_kind: None,
            urls: vec![],
            composer: crate::default(),
            npm: npm_context(&npmrc_texts, &yarnrc_texts, &bunfig_texts, &env),
            dotnet: crate::default(),
            maven: crate::default(),
            cargo_registries: vec![],
            python_sources: vec![],
            go: crate::default(),
        }
    }
}

include!("context_sources.rs");
include!("context_helpers.rs");
