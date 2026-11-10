use serde::{Deserialize, Serialize};

/// A `NuGet` package source as listed by `dotnet nuget list source`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetSource {
    /// Whether the source is enabled.
    pub enabled: bool,
    /// Whether the source is configured machine-wide.
    pub machine_wide: bool,
    /// The source URL.
    pub url: String,
    /// The URL protocol, such as `https:`.
    pub protocol: String,
}

/// The credentials that a `NuGet.Config` holds for a source.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetAuthEntry {
    /// The registry URL.
    pub registry: String,
    /// The `Authorization` header value to send.
    pub header_value: String,
}

/// A named package source of a `NuGet.Config`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetNamedSource {
    /// The source name.
    pub name: String,
    /// The source URL.
    pub url: String,
}

/// A package source mapping of a `NuGet.Config`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetSourceMapping {
    /// The name of the source that the pattern maps to.
    pub source: String,
    /// The package ID pattern of the mapping.
    pub pattern: String,
}

/// The sources, credentials and mappings of a `NuGet.Config`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetNugetConfig {
    /// The package sources.
    pub sources: Vec<DotnetNamedSource>,
    /// The credentials per source.
    pub auth_entries: Vec<DotnetAuthEntry>,
    /// The package source mappings.
    pub source_mappings: Vec<DotnetSourceMapping>,
    /// The names of the sources that the file removes.
    pub removed_sources: Vec<String>,
    /// The names of the source mappings that the file removes.
    pub removed_source_mappings: Vec<String>,
    /// Whether the file clears the sources of earlier files.
    pub clear_sources: bool,
    /// Whether the file clears the credentials of earlier files.
    pub clear_auth_entries: bool,
    /// Whether the file clears the source mappings of earlier files.
    pub clear_source_mappings: bool,
}
