use serde::{Deserialize, Serialize};
use versionlens_cache::minutes_to_ms;
use versionlens_model::ManifestKind::{
    AnsibleGalaxyRequirementsYaml, BazelModule, BazelWorkspace, Cabal, CabalProject, CargoToml,
    ClojureDepsEdn, Cmake, CocoaPodsPodfile, ComposerJson, ConanfilePy, ConanfileTxt, Cpanfile,
    DenoImportMapJson, DenoJson, DockerComposeYaml, DotnetXml, DubJson, DuneProject, Gemfile,
    GleamToml, GoMod, GradleBuild, GradleSettings, GradleVersionCatalogToml, HaxelibJson,
    HelmChartYaml, JsrJson, JuliaManifestToml, JuliaProjectToml, KustomizationYaml,
    LeiningenProjectClj, LuaRockspec, MavenPomXml, MesonWrap, MixExs, Nimble, NixFlake,
    NpmPackageJson, NpmPackageJson5, NpmPackageYaml, Opam, PaketDependencies, PaketReferences,
    PnpmYaml, PubspecYaml, PythonRequirementsTxt, RDescription, RebarConfig, RenvLock, RubyGemspec,
    SbtBuild, StackYaml, SwiftPackage, TerraformTf, UnityProjectManifestJson, VcpkgJson, XmakeLua,
    ZigBuildZon,
};
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

#[derive(Debug, Default, PartialEq)]
pub struct ProviderSettingsInput {
    pub registry_urls: Option<Vec<RegistryUrlConfigInput>>,
    pub prerelease_tag_filters: Option<Vec<PrereleaseTagConfigInput>>,
    pub provider_cache: Option<Vec<ProviderCacheConfigInput>>,
    pub provider_http: Option<Vec<ProviderHttpConfigInput>>,
    pub dependency_properties: Option<Vec<DependencyPropertyConfigInput>>,
    pub file_patterns: Option<Vec<FilePatternConfigInput>>,
}

fn registry_url_config_from_input(input: RegistryUrlConfigInput) -> Option<RegistryUrlConfig> {
    input.try_into().ok()
}

fn prerelease_tag_config_from_input(
    input: PrereleaseTagConfigInput,
) -> Option<PrereleaseTagConfig> {
    input.try_into().ok()
}

fn provider_http_config_from_input(input: ProviderHttpConfigInput) -> Option<ProviderHttpConfig> {
    input.try_into().ok()
}

fn dependency_property_config_from_input(
    input: DependencyPropertyConfigInput,
) -> Option<DependencyPropertyConfig> {
    input.try_into().ok()
}

fn file_pattern_config_from_input(input: FilePatternConfigInput) -> Option<FilePatternConfig> {
    input.try_into().ok()
}

impl ProviderSettings {
    pub fn from_input(input: ProviderSettingsInput) -> Self {
        Self {
            registry_urls: input
                .registry_urls
                .unwrap_or_default()
                .into_iter()
                .filter_map(registry_url_config_from_input)
                .collect(),
            prerelease_tags: input
                .prerelease_tag_filters
                .unwrap_or_default()
                .into_iter()
                .filter_map(prerelease_tag_config_from_input)
                .collect(),
            provider_cache: input
                .provider_cache
                .unwrap_or_default()
                .into_iter()
                .filter_map(provider_cache_config_from_input)
                .collect(),
            provider_http: input
                .provider_http
                .unwrap_or_default()
                .into_iter()
                .filter_map(provider_http_config_from_input)
                .collect(),
            dependency_properties: input
                .dependency_properties
                .unwrap_or_default()
                .into_iter()
                .filter_map(dependency_property_config_from_input)
                .collect(),
            file_patterns: input
                .file_patterns
                .unwrap_or_default()
                .into_iter()
                .filter_map(file_pattern_config_from_input)
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct FilePatternConfigInput {
    pub ecosystem: String,
    pub pattern: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePatternConfig {
    pub manifest_kind: ManifestKind,
    pub pattern: String,
}

impl FilePatternConfig {
    pub fn from_input(input: FilePatternConfigInput) -> Option<Self> {
        file_pattern_config_from_name(&input.ecosystem, input.pattern)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DependencyPropertyConfigInput {
    pub ecosystem: String,
    pub provider: Option<String>,
    pub properties: Vec<String>,
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

    pub fn from_input(input: DependencyPropertyConfigInput) -> Option<Self> {
        dependency_property_config_from_name(
            &input.ecosystem,
            input.provider.as_deref(),
            input.properties,
        )
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
        || (candidate == NpmPackageJson && matches!(kind, NpmPackageJson5 | NpmPackageYaml))
        || (candidate == DenoJson && matches!(kind, DenoImportMapJson | JsrJson))
        || (candidate == DotnetXml && matches!(kind, PaketDependencies | PaketReferences))
        || (candidate == Gemfile && kind == RubyGemspec)
        || (candidate == MavenPomXml
            && matches!(
                kind,
                GradleBuild
                    | GradleSettings
                    | GradleVersionCatalogToml
                    | SbtBuild
                    | ClojureDepsEdn
                    | LeiningenProjectClj
            ))
        || (candidate == Cabal && matches!(kind, CabalProject | StackYaml))
        || (candidate == JuliaProjectToml && kind == JuliaManifestToml)
        || (candidate == RDescription && kind == RenvLock)
        || (candidate == Opam && kind == DuneProject)
        || (candidate == ConanfileTxt && kind == ConanfilePy)
        || (candidate == Cmake && matches!(kind, XmakeLua | MesonWrap | BazelWorkspace))
}

pub fn enabled_provider_config_from_name(provider: &str) -> Option<EnabledProviderConfig> {
    Some(EnabledProviderConfig {
        ecosystem: ecosystem_from_config_name(provider)?,
        manifest_kind: enabled_provider_manifest_kind_from_name(provider),
    })
}

fn enabled_provider_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "npm" | "bun" => Some(NpmPackageJson),
        "pnpm" => Some(PnpmYaml),
        "kustomize" => Some(KustomizationYaml),
        "unity" => Some(UnityProjectManifestJson),
        "cocoapods" => Some(CocoaPodsPodfile),
        _ => None,
    }
}

pub fn dependency_property_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "npm" | "bun" => Some(NpmPackageJson),
        "pnpm" => Some(PnpmYaml),
        "kustomize" => Some(KustomizationYaml),
        "unity" => Some(UnityProjectManifestJson),
        "cocoapods" => Some(CocoaPodsPodfile),
        _ => None,
    }
}

pub fn dependency_property_config_from_name(
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

pub fn provider_settings_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "pnpm" => Some(PnpmYaml),
        "kustomize" => Some(KustomizationYaml),
        "unity" => Some(UnityProjectManifestJson),
        "cocoapods" => Some(CocoaPodsPodfile),
        _ => None,
    }
}

pub fn file_pattern_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "cargo" => Some(CargoToml),
        "composer" => Some(ComposerJson),
        "deno" => Some(DenoJson),
        "docker" => Some(DockerComposeYaml),
        "kustomize" => Some(KustomizationYaml),
        "unity" => Some(UnityProjectManifestJson),
        "dotnet" => Some(DotnetXml),
        "dub" => Some(DubJson),
        "go" | "golang" => Some(GoMod),
        "hex" | "beam" => Some(MixExs),
        "gleam" => Some(GleamToml),
        "rebar" => Some(RebarConfig),
        "opam" | "ocaml" => Some(Opam),
        "dune" => Some(DuneProject),
        "hackage" | "haskell" => Some(Cabal),
        "stack" => Some(StackYaml),
        "julia" => Some(JuliaProjectToml),
        "julia-manifest" => Some(JuliaManifestToml),
        "cran" | "r" => Some(RDescription),
        "renv" => Some(RenvLock),
        "conan" => Some(ConanfileTxt),
        "vcpkg" => Some(VcpkgJson),
        "cpp" | "c-cpp" | "cmake" => Some(Cmake),
        "xmake" => Some(XmakeLua),
        "meson" => Some(MesonWrap),
        "bazel-workspace" => Some(BazelWorkspace),
        "swift" => Some(SwiftPackage),
        "zig" => Some(ZigBuildZon),
        "nim" => Some(Nimble),
        "luarocks" | "lua" => Some(LuaRockspec),
        "cpan" | "perl" => Some(Cpanfile),
        "haxelib" | "haxe" => Some(HaxelibJson),
        "terraform" | "opentofu" | "tofu" => Some(TerraformTf),
        "helm" => Some(HelmChartYaml),
        "ansible" | "ansible-galaxy" | "galaxy" => Some(AnsibleGalaxyRequirementsYaml),
        "bazel" | "bzlmod" => Some(BazelModule),
        "nix" => Some(NixFlake),
        "cocoapods" => Some(CocoaPodsPodfile),
        "gradle" | "gradle-build" => Some(GradleBuild),
        "gradle-settings" => Some(GradleSettings),
        "gradle-version-catalog" => Some(GradleVersionCatalogToml),
        "sbt" | "scala" => Some(SbtBuild),
        "clojure" | "deps-edn" => Some(ClojureDepsEdn),
        "leiningen" | "lein" | "project-clj" => Some(LeiningenProjectClj),
        "maven" => Some(MavenPomXml),
        "bun" | "npm" => Some(NpmPackageJson),
        "pnpm" => Some(PnpmYaml),
        "pypi" | "python" => Some(PythonRequirementsTxt),
        "pub" => Some(PubspecYaml),
        "ruby" => Some(Gemfile),
        _ => None,
    }
}

pub fn file_pattern_config_from_name(
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

pub fn registry_url_config_from_name(ecosystem: &str, url: String) -> Option<RegistryUrlConfig> {
    let url = url.trim();
    if url.is_empty() {
        return None;
    }

    Some(RegistryUrlConfig {
        ecosystem: ecosystem_from_config_name(ecosystem)?,
        url: url.to_owned(),
    })
}

#[derive(Debug, PartialEq, Eq)]
pub struct RegistryUrlConfigInput {
    pub ecosystem: String,
    pub url: String,
}

impl RegistryUrlConfig {
    pub fn from_input(input: RegistryUrlConfigInput) -> Option<Self> {
        registry_url_config_from_name(&input.ecosystem, input.url)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PrereleaseTagConfigInput {
    pub ecosystem: String,
    pub tags: Vec<String>,
}

impl PrereleaseTagConfig {
    pub fn from_input(input: PrereleaseTagConfigInput) -> Option<Self> {
        prerelease_tag_config_from_name(&input.ecosystem, input.tags)
    }
}

pub fn prerelease_tag_config_from_name(
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

#[derive(Debug, PartialEq)]
pub struct ProviderCacheConfigInput {
    pub ecosystem: String,
    pub cache_duration_minutes: Option<f64>,
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

pub fn provider_cache_config_from_input(
    input: ProviderCacheConfigInput,
) -> Option<ProviderCacheConfig> {
    provider_cache_config_from_name(&input.ecosystem, input.cache_duration_minutes)
}

pub fn provider_cache_config_from_name(
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

#[derive(Debug, PartialEq, Eq)]
pub struct ProviderHttpConfigInput {
    pub ecosystem: String,
    pub strict_ssl: Option<bool>,
}

impl ProviderHttpConfig {
    pub fn from_input(input: ProviderHttpConfigInput) -> Option<Self> {
        provider_http_config_from_name(&input.ecosystem, input.strict_ssl)
    }
}

pub fn provider_http_config_from_name(
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

impl TryFrom<RegistryUrlConfigInput> for RegistryUrlConfig {
    type Error = ();

    fn try_from(input: RegistryUrlConfigInput) -> Result<Self, Self::Error> {
        Self::from_input(input).ok_or(())
    }
}

impl TryFrom<PrereleaseTagConfigInput> for PrereleaseTagConfig {
    type Error = ();

    fn try_from(input: PrereleaseTagConfigInput) -> Result<Self, Self::Error> {
        Self::from_input(input).ok_or(())
    }
}

impl TryFrom<ProviderHttpConfigInput> for ProviderHttpConfig {
    type Error = ();

    fn try_from(input: ProviderHttpConfigInput) -> Result<Self, Self::Error> {
        Self::from_input(input).ok_or(())
    }
}

impl TryFrom<DependencyPropertyConfigInput> for DependencyPropertyConfig {
    type Error = ();

    fn try_from(input: DependencyPropertyConfigInput) -> Result<Self, Self::Error> {
        Self::from_input(input).ok_or(())
    }
}

impl TryFrom<FilePatternConfigInput> for FilePatternConfig {
    type Error = ();

    fn try_from(input: FilePatternConfigInput) -> Result<Self, Self::Error> {
        Self::from_input(input).ok_or(())
    }
}
