use serde::{Deserialize, Serialize};

use crate::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyPayload {
    pub name: String,
    pub requirement: String,
    pub ecosystem: String,
    pub group: String,
    pub hosted_url: Option<String>,
    pub hosted_name: Option<String>,
    pub range: Range,
    pub requirement_range: Range,
}
