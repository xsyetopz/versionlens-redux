use serde::{Deserialize, Serialize};
use versionlens_vscode_model::Range;

use super::Ecosystem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub name: String,
    pub requirement: String,
    pub ecosystem: Ecosystem,
    pub group: String,
    pub hosted_url: Option<String>,
    pub hosted_name: Option<String>,
    pub range: Range,
    pub requirement_range: Range,
    pub requirement_prefix: String,
    pub requirement_suffix: String,
}
