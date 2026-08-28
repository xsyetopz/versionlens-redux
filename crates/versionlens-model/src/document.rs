use serde::{Deserialize, Serialize};

use crate::{Ecosystem, Range};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
}

impl DocumentInput {
    pub fn new(
        uri: impl Into<String>,
        language_id: impl Into<String>,
        text: impl Into<String>,
        workspace_root: Option<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            language_id: language_id.into(),
            text: text.into(),
            workspace_root,
        }
    }
}

pub fn registry_alias_requirement(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let Some(split) = spec.rfind('@').filter(|index| *index > 0) else {
        return Some("");
    };
    Some(&spec[split + 1..])
}

pub fn is_npm_dist_tag_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim();
    !requirement.is_empty()
        && requirement
            .chars()
            .any(|character| character.is_ascii_alphabetic())
        && requirement.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
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
