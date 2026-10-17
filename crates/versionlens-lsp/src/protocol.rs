//! Wire types for the `VersionLens` extensions to the Language Server Protocol.
//!
//! Every type here is serialized with camelCase field names. Command arguments
//! are named objects, never positional lists.

use lsp_types::notification::Notification;
use lsp_types::request::Request;
use lsp_types::{Range, Uri};
use serde::{Deserialize, Serialize};

/// Busy and error state of one document, sent by `versionlens/status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Idle,
    Busy,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusParams {
    pub uri: Uri,
    pub status: DocumentStatus,
}

/// Server-to-client notification `versionlens/status`.
pub enum StatusNotification {}

impl Notification for StatusNotification {
    type Params = StatusParams;
    const METHOD: &'static str = "versionlens/status";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationParams {
    pub host: String,
    pub status: u16,
}

/// Credential returned by the client. Deliberately not `Debug`, so the token
/// cannot reach a log through this type.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationCredential {
    pub scheme: String,
    pub token: String,
}

/// Server-to-client request `versionlens/authorization`. The client answers
/// with a credential, or `null` when it has none.
pub enum AuthorizationRequest {}

impl Request for AuthorizationRequest {
    type Params = AuthorizationParams;
    type Result = Option<AuthorizationCredential>;
    const METHOD: &'static str = "versionlens/authorization";
}

/// Bulk update level, as named in `versionlens.editor.onUpdateDependencies*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateLevel {
    Latest,
    Major,
    Minor,
    Patch,
}

/// Single-dependency update command. The values are the same command names
/// that version lenses already carry, including the project-version bumps
/// `updateRelease` and `updatePrerelease`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpdateCommand {
    Update,
    UpdateMajor,
    UpdateMinor,
    UpdatePatch,
    UpdateRelease,
    UpdatePrerelease,
}

/// Arguments for commands that act on one whole document, such as sort
/// dependencies and query manifest information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentArguments {
    pub uri: Uri,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUpdateArguments {
    pub uri: Uri,
    pub level: UpdateLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDependencyArguments {
    pub uri: Uri,
    pub name: String,
    pub requirement_range: Range,
    pub command: UpdateCommand,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChooseBuildArguments {
    pub uri: Uri,
    pub name: String,
    pub requirement_range: Range,
    pub requirement: String,
    pub builds: Vec<String>,
}

#[cfg(test)]
mod tests;
