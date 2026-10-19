use serde::{Deserialize, Serialize};
use versionlens_fetch::minutes_to_ms;

use versionlens_model::{Ecosystem, ManifestKind, ecosystem_from_config_name};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSettings {
    pub registry_urls: Vec<RegistryUrlConfig>,
    pub prerelease_tags: Vec<PrereleaseTagConfig>,
    pub provider_cache: Vec<ProviderCacheConfig>,
    pub provider_http: Vec<ProviderHttpConfig>,
    pub dependency_properties: Vec<DependencyPropertyConfig>,
    pub file_patterns: Vec<FilePatternConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePatternConfig {
    pub manifest_kind: ManifestKind,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyPropertyConfig {
    pub ecosystem: Ecosystem,
    pub manifest_kind: Option<ManifestKind>,
    pub properties: Vec<String>,
}

impl DependencyPropertyConfig {
    pub(crate) fn applies_to_manifest(&self, kind: ManifestKind) -> bool {
        self.manifest_kind
            .is_none_or(|candidate| manifest_kind_matches(candidate, kind))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnabledProviderConfig {
    pub ecosystem: Ecosystem,
    pub manifest_kind: Option<ManifestKind>,
}

impl EnabledProviderConfig {
    pub(crate) fn applies_to_manifest(&self, kind: ManifestKind, ecosystem: Ecosystem) -> bool {
        self.ecosystem == ecosystem
            && self
                .manifest_kind
                .is_none_or(|candidate| manifest_kind_matches(candidate, kind))
    }
}

fn manifest_kind_matches(candidate: ManifestKind, kind: ManifestKind) -> bool {
    candidate == kind
        || (candidate == ManifestKind::NpmPackageJson
            && matches!(
                kind,
                ManifestKind::NpmPackageJson5
                    | ManifestKind::NpmPackageYaml
                    | ManifestKind::Nvmrc
                    | ManifestKind::NodeVersion
                    | ManifestKind::BunVersion
            ))
        || (candidate == ManifestKind::CargoToml
            && matches!(
                kind,
                ManifestKind::RustToolchain | ManifestKind::RustToolchainToml
            ))
        || (candidate == ManifestKind::DenoJson
            && matches!(
                kind,
                ManifestKind::DenoImportMapJson | ManifestKind::JsrJson
            ))
        || (candidate == ManifestKind::DotnetXml
            && matches!(
                kind,
                ManifestKind::PaketDependencies | ManifestKind::PaketReferences
            ))
        || (candidate == ManifestKind::Gemfile && kind == ManifestKind::RubyGemspec)
        || (candidate == ManifestKind::MavenPomXml
            && matches!(
                kind,
                ManifestKind::GradleBuild
                    | ManifestKind::GradleSettings
                    | ManifestKind::GradleVersionCatalogToml
                    | ManifestKind::SbtBuild
                    | ManifestKind::ClojureDepsEdn
                    | ManifestKind::LeiningenProjectClj
            ))
        || (candidate == ManifestKind::Cabal
            && matches!(kind, ManifestKind::CabalProject | ManifestKind::StackYaml))
        || (candidate == ManifestKind::JuliaProjectToml && kind == ManifestKind::JuliaManifestToml)
        || (candidate == ManifestKind::RDescription && kind == ManifestKind::RenvLock)
        || (candidate == ManifestKind::Opam && kind == ManifestKind::DuneProject)
        || (candidate == ManifestKind::ConanfileTxt && kind == ManifestKind::ConanfilePy)
        || (candidate == ManifestKind::Cmake
            && matches!(
                kind,
                ManifestKind::XmakeLua | ManifestKind::MesonWrap | ManifestKind::BazelWorkspace
            ))
}

pub(crate) fn enabled_provider_config_from_name(provider: &str) -> Option<EnabledProviderConfig> {
    Some(EnabledProviderConfig {
        ecosystem: ecosystem_from_config_name(provider)?,
        manifest_kind: dependency_property_manifest_kind_from_name(provider),
    })
}

pub(crate) fn dependency_property_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "npm" | "bun" => Some(ManifestKind::NpmPackageJson),
        _ => provider_settings_manifest_kind_from_name(provider),
    }
}

pub(crate) fn dependency_property_config_from_name(
    ecosystem: &str,
    provider: Option<&str>,
    properties: Vec<String>,
) -> Option<DependencyPropertyConfig> {
    let provider = provider.unwrap_or(ecosystem);
    Some(DependencyPropertyConfig {
        ecosystem: ecosystem_from_config_name(ecosystem)?,
        manifest_kind: dependency_property_manifest_kind_from_name(provider),
        properties,
    })
}

pub(crate) fn provider_settings_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "pnpm" => Some(ManifestKind::PnpmYaml),
        "kustomize" => Some(ManifestKind::KustomizationYaml),
        "unity" => Some(ManifestKind::UnityProjectManifestJson),
        "cocoapods" => Some(ManifestKind::CocoaPodsPodfile),
        "github" => Some(ManifestKind::GitHubActions),
        _ => None,
    }
}

pub(crate) fn file_pattern_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "cargo" => Some(ManifestKind::CargoToml),
        "composer" => Some(ManifestKind::ComposerJson),
        "deno" => Some(ManifestKind::DenoJson),
        "docker" => Some(ManifestKind::DockerComposeYaml),
        "kustomize" => Some(ManifestKind::KustomizationYaml),
        "unity" => Some(ManifestKind::UnityProjectManifestJson),
        "dotnet" => Some(ManifestKind::DotnetXml),
        "dub" => Some(ManifestKind::DubJson),
        "go" | "golang" => Some(ManifestKind::GoMod),
        "hex" | "beam" => Some(ManifestKind::MixExs),
        "gleam" => Some(ManifestKind::GleamToml),
        "rebar" => Some(ManifestKind::RebarConfig),
        "opam" | "ocaml" => Some(ManifestKind::Opam),
        "dune" => Some(ManifestKind::DuneProject),
        "hackage" | "haskell" => Some(ManifestKind::Cabal),
        "stack" => Some(ManifestKind::StackYaml),
        "julia" => Some(ManifestKind::JuliaProjectToml),
        "julia-manifest" => Some(ManifestKind::JuliaManifestToml),
        "cran" | "r" => Some(ManifestKind::RDescription),
        "renv" => Some(ManifestKind::RenvLock),
        "conan" => Some(ManifestKind::ConanfileTxt),
        "vcpkg" => Some(ManifestKind::VcpkgJson),
        "cpp" | "c-cpp" | "cmake" => Some(ManifestKind::Cmake),
        "xmake" => Some(ManifestKind::XmakeLua),
        "meson" => Some(ManifestKind::MesonWrap),
        "bazel-workspace" => Some(ManifestKind::BazelWorkspace),
        "swift" => Some(ManifestKind::SwiftPackage),
        "zig" => Some(ManifestKind::ZigBuildZon),
        "nim" => Some(ManifestKind::Nimble),
        "luarocks" | "lua" => Some(ManifestKind::LuaRockspec),
        "cpan" | "perl" => Some(ManifestKind::Cpanfile),
        "haxelib" | "haxe" => Some(ManifestKind::HaxelibJson),
        "terraform" | "opentofu" | "tofu" => Some(ManifestKind::TerraformTf),
        "helm" => Some(ManifestKind::HelmChartYaml),
        "ansible" | "ansible-galaxy" | "galaxy" => {
            Some(ManifestKind::AnsibleGalaxyRequirementsYaml)
        }
        "bazel" | "bzlmod" => Some(ManifestKind::BazelModule),
        "nix" => Some(ManifestKind::NixFlake),
        "cocoapods" => Some(ManifestKind::CocoaPodsPodfile),
        "github" => Some(ManifestKind::GitHubActions),
        "gradle" | "gradle-build" => Some(ManifestKind::GradleBuild),
        "gradle-settings" => Some(ManifestKind::GradleSettings),
        "gradle-version-catalog" => Some(ManifestKind::GradleVersionCatalogToml),
        "sbt" | "scala" => Some(ManifestKind::SbtBuild),
        "clojure" | "deps-edn" => Some(ManifestKind::ClojureDepsEdn),
        "leiningen" | "lein" | "project-clj" => Some(ManifestKind::LeiningenProjectClj),
        "maven" => Some(ManifestKind::MavenPomXml),
        "bun" | "npm" => Some(ManifestKind::NpmPackageJson),
        "pnpm" => Some(ManifestKind::PnpmYaml),
        "pypi" | "python" => Some(ManifestKind::PythonRequirementsTxt),
        "pub" => Some(ManifestKind::PubspecYaml),
        "ruby" => Some(ManifestKind::Gemfile),
        _ => None,
    }
}

pub(crate) fn file_pattern_config_from_name(
    ecosystem: &str,
    pattern: String,
) -> Option<FilePatternConfig> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return None;
    }

    Some(FilePatternConfig {
        manifest_kind: file_pattern_manifest_kind_from_name(ecosystem)?,
        pattern: pattern.to_owned(),
    })
}

pub(crate) fn registry_url_config_from_name(
    ecosystem: &str,
    url: String,
) -> Option<RegistryUrlConfig> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    Some(RegistryUrlConfig {
        ecosystem: ecosystem_from_config_name(ecosystem)?,
        url: url.to_owned(),
    })
}

pub(crate) fn prerelease_tag_config_from_name(
    ecosystem: &str,
    tags: Vec<String>,
) -> Option<PrereleaseTagConfig> {
    let ecosystem = ecosystem_from_config_name(ecosystem)?;
    let tags = tags
        .into_iter()
        .map(|tag| tag.trim().to_owned())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();

    if tags.is_empty() {
        return None;
    }

    Some(PrereleaseTagConfig { ecosystem, tags })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCacheConfig {
    pub ecosystem: Ecosystem,
    pub manifest_kind: Option<ManifestKind>,
    pub cache_ttl_ms: u64,
}

impl ProviderCacheConfig {
    pub(crate) fn applies_to_manifest(&self, kind: Option<ManifestKind>) -> bool {
        self.manifest_kind
            .is_none_or(|candidate| kind == Some(candidate))
    }
}

pub(crate) fn provider_cache_config_from_name(
    ecosystem: &str,
    cache_duration_minutes: Option<f64>,
) -> Option<ProviderCacheConfig> {
    let minutes = cache_duration_minutes?;
    Some(ProviderCacheConfig {
        ecosystem: ecosystem_from_config_name(ecosystem)?,
        manifest_kind: provider_settings_manifest_kind_from_name(ecosystem),
        cache_ttl_ms: minutes_to_ms(minutes)?,
    })
}

pub(crate) fn provider_http_config_from_name(
    ecosystem: &str,
    strict_ssl: Option<bool>,
) -> Option<ProviderHttpConfig> {
    Some(ProviderHttpConfig {
        ecosystem: ecosystem_from_config_name(ecosystem)?,
        manifest_kind: provider_settings_manifest_kind_from_name(ecosystem),
        strict_ssl,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryUrlConfig {
    pub ecosystem: Ecosystem,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrereleaseTagConfig {
    pub ecosystem: Ecosystem,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderHttpConfig {
    pub ecosystem: Ecosystem,
    pub manifest_kind: Option<ManifestKind>,
    pub strict_ssl: Option<bool>,
}

impl ProviderHttpConfig {
    pub(crate) fn applies_to_manifest(&self, kind: Option<ManifestKind>) -> bool {
        self.manifest_kind
            .is_none_or(|candidate| kind == Some(candidate))
    }
}
