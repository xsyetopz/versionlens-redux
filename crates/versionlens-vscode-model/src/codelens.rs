use serde::{Deserialize, Serialize};

use versionlens_model::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeLensPayload {
    pub range: Range,
    pub title: String,
    pub command: String,
    pub arguments: Vec<String>,
}
