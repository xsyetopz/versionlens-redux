use serde::{Deserialize, Serialize};
use versionlens_cache::minutes_to_ms;
use versionlens_parsers::{Ecosystem, ManifestKind, ecosystem_from_config_name};

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

impl ProviderSettings {
    pub fn from_input(input: ProviderSettingsInput) -> Self {
        Self {
            registry_urls: input
                .registry_urls
                .unwrap_or_default()
                .into_iter()
                .filter_map(RegistryUrlConfig::from_input)
                .collect(),
            prerelease_tags: input
                .prerelease_tag_filters
                .unwrap_or_default()
                .into_iter()
                .filter_map(PrereleaseTagConfig::from_input)
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
                .filter_map(ProviderHttpConfig::from_input)
                .collect(),
            dependency_properties: input
                .dependency_properties
                .unwrap_or_default()
                .into_iter()
                .filter_map(DependencyPropertyConfig::from_input)
                .collect(),
            file_patterns: input
                .file_patterns
                .unwrap_or_default()
                .into_iter()
                .filter_map(FilePatternConfig::from_input)
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
        self.manifest_kind.is_none_or(|candidate| candidate == kind)
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
        self.ecosystem == ecosystem && self.manifest_kind.is_none_or(|candidate| candidate == kind)
    }
}

pub fn enabled_provider_config_from_name(provider: &str) -> Option<EnabledProviderConfig> {
    Some(EnabledProviderConfig {
        ecosystem: ecosystem_from_config_name(provider)?,
        manifest_kind: enabled_provider_manifest_kind_from_name(provider),
    })
}

fn enabled_provider_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "npm" | "bun" => Some(ManifestKind::NpmPackageJson),
        "pnpm" => Some(ManifestKind::PnpmYaml),
        _ => None,
    }
}

pub fn dependency_property_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "npm" | "bun" => Some(ManifestKind::NpmPackageJson),
        "pnpm" => Some(ManifestKind::PnpmYaml),
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
        "pnpm" => Some(ManifestKind::PnpmYaml),
        _ => None,
    }
}

pub fn file_pattern_manifest_kind_from_name(provider: &str) -> Option<ManifestKind> {
    match provider {
        "cargo" => Some(ManifestKind::CargoToml),
        "composer" => Some(ManifestKind::ComposerJson),
        "deno" => Some(ManifestKind::DenoJson),
        "docker" => Some(ManifestKind::DockerComposeYaml),
        "dotnet" => Some(ManifestKind::DotnetXml),
        "dub" => Some(ManifestKind::DubJson),
        "go" | "golang" => Some(ManifestKind::GoMod),
        "maven" => Some(ManifestKind::MavenPomXml),
        "bun" | "npm" => Some(ManifestKind::NpmPackageJson),
        "pnpm" => Some(ManifestKind::PnpmYaml),
        "pypi" | "python" => Some(ManifestKind::PythonRequirementsTxt),
        "pub" => Some(ManifestKind::PubspecYaml),
        "ruby" => Some(ManifestKind::Gemfile),
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
