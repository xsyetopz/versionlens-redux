use serde::{Deserialize, Serialize};
use versionlens_model::Dependency;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// How a dependency's requirement compares with what the registry offers.
pub enum SuggestionStatus {
    /// A build of the current version is available.
    BuildAvailable,
    /// The requirement is already at the latest version.
    Current,
    /// The dependency points at a local directory.
    Directory,
    /// The dependency points at a local directory that does not exist.
    DirectoryNotFound,
    /// Resolving the dependency failed.
    Error,
    /// The dependency is pinned to a fixed source such as a git repository.
    Fixed,
    /// The dependency is not valid.
    Invalid,
    /// The requirement is not a valid range.
    InvalidRange,
    /// The registry has no matching version.
    NoMatch,
    /// The dependency kind is not supported.
    NotSupported,
    /// The host refused the request until its rate limit resets.
    RateLimited,
    /// The requirement accepts a version but not the latest one.
    Satisfies,
    /// The requirement accepts the latest version.
    SatisfiesLatest,
    /// No latest version has been resolved yet.
    Unresolved,
    /// A newer version is available.
    UpdateAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The resolved state of one dependency and the update choices it offers.
pub struct Suggestion {
    /// The dependency this suggestion describes.
    pub dependency: Dependency,
    /// The latest version, or a message shown in its place for some statuses.
    pub latest: Option<String>,
    /// The resolved path or version, when the status has one.
    pub resolved: Option<String>,
    /// How the requirement compares with the registry.
    pub status: SuggestionStatus,
    /// Build identifiers available for the current version.
    pub builds: Vec<String>,
    /// The update choices offered for the dependency.
    pub choices: Vec<UpdateChoice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// One update a user can pick for a dependency.
pub struct UpdateChoice {
    /// The text shown for the choice.
    pub label: String,
    /// The version the choice updates to.
    pub version: String,
    /// The text that replaces the requirement, when it differs from `version`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    /// The command that applies the choice.
    pub command: String,
}
