use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusPayload {
    pub dependency_count: u32,
    pub update_count: u32,
    pub vulnerability_count: u32,
    pub error_count: u32,
    pub no_match_count: u32,
    pub visible: bool,
    pub text: String,
    pub tooltip: String,
}
