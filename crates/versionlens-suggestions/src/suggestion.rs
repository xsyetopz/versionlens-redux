use serde::{Deserialize, Serialize};
use versionlens_model::Dependency;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SuggestionStatus {
    BuildAvailable,
    Current,
    Directory,
    DirectoryNotFound,
    Error,
    Fixed,
    Invalid,
    InvalidRange,
    NoMatch,
    NotSupported,
    Satisfies,
    SatisfiesLatest,
    Unresolved,
    UpdateAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Suggestion {
    pub dependency: Dependency,
    pub latest: Option<String>,
    pub resolved: Option<String>,
    pub status: SuggestionStatus,
    pub builds: Vec<String>,
    pub choices: Vec<UpdateChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChoice {
    pub label: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    pub command: String,
}
