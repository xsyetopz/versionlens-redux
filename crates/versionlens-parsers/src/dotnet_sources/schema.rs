use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetSource {
    pub enabled: bool,
    pub machine_wide: bool,
    pub url: String,
    pub protocol: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetAuthEntry {
    pub registry: String,
    pub header_value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetNamedSource {
    pub name: String,
    pub url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetSourceMapping {
    pub source: String,
    pub pattern: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DotnetNugetConfig {
    pub sources: Vec<DotnetNamedSource>,
    pub auth_entries: Vec<DotnetAuthEntry>,
    pub source_mappings: Vec<DotnetSourceMapping>,
    pub removed_sources: Vec<String>,
    pub removed_source_mappings: Vec<String>,
    pub clear_sources: bool,
    pub clear_auth_entries: bool,
    pub clear_source_mappings: bool,
}
