use serde::{Deserialize, Serialize};

use crate::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticPayload {
    pub range: Range,
    pub message: String,
    pub severity: u8,
    pub source: Option<String>,
    pub code: Option<String>,
    pub code_description_url: Option<String>,
}
