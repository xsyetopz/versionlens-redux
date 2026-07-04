use serde::{Deserialize, Serialize};

use crate::DependencyPayload;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionPayload {
    pub dependency: DependencyPayload,
    pub latest: Option<String>,
    pub status: String,
    pub builds: Vec<String>,
}
