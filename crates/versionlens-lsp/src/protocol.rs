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
mod tests {
use std::fmt::Debug;

use lsp_types::notification::Notification;
use lsp_types::request::Request;
use lsp_types::{Position, Range, Uri};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::{
    AuthorizationCredential, AuthorizationParams, AuthorizationRequest, BulkUpdateArguments,
    ChooseBuildArguments, DocumentArguments, DocumentStatus, StatusNotification, StatusParams,
    UpdateCommand, UpdateDependencyArguments, UpdateLevel,
};

type TestResult = Result<(), Box<dyn std::error::Error>>;

const DOCUMENT_URI: &str = "file:///workspace/package.json";

fn document_uri() -> Result<Uri, Box<dyn std::error::Error>> {
    Ok(DOCUMENT_URI.parse::<Uri>()?)
}

fn requirement_range() -> Range {
    Range::new(Position::new(3, 12), Position::new(3, 18))
}

fn assert_round_trip<T>(value: &T, expected: &Value) -> TestResult
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_eq!(serde_json::to_value(value)?, *expected);
    assert_eq!(serde_json::from_value::<T>(expected.clone())?, *value);
    Ok(())
}

#[test]
fn protocol_status_notification_uses_documented_method_and_states() -> TestResult {
    assert_eq!(StatusNotification::METHOD, "versionlens/status");
    for (status, state) in [
        (DocumentStatus::Idle, "idle"),
        (DocumentStatus::Busy, "busy"),
        (DocumentStatus::Error, "error"),
    ] {
        assert_round_trip(
            &StatusParams {
                uri: document_uri()?,
                status,
            },
            &json!({"uri": DOCUMENT_URI, "status": state}),
        )?;
    }
    assert!(
        serde_json::from_value::<StatusParams>(json!({"uri": DOCUMENT_URI, "status": "done"}))
            .is_err()
    );
    Ok(())
}

#[test]
fn protocol_authorization_request_round_trips_params_and_credential() -> TestResult {
    assert_eq!(AuthorizationRequest::METHOD, "versionlens/authorization");
    assert_round_trip(
        &AuthorizationParams {
            host: "api.github.com".to_owned(),
            status: 401,
        },
        &json!({"host": "api.github.com", "status": 401}),
    )?;

    let credential = AuthorizationCredential {
        scheme: "Bearer".to_owned(),
        token: "test-token".to_owned(),
    };
    let credential_json = json!({"scheme": "Bearer", "token": "test-token"});
    assert_eq!(serde_json::to_value(Some(&credential))?, credential_json);
    let decoded: Option<AuthorizationCredential> = serde_json::from_value(credential_json)?;
    assert_eq!(
        decoded.map(|credential| (credential.scheme, credential.token)),
        Some(("Bearer".to_owned(), "test-token".to_owned()))
    );
    Ok(())
}

#[test]
fn protocol_authorization_declined_credential_is_null() -> TestResult {
    assert_eq!(
        serde_json::to_value(None::<AuthorizationCredential>)?,
        Value::Null
    );
    let declined: Option<AuthorizationCredential> = serde_json::from_value(Value::Null)?;
    assert!(declined.is_none());
    Ok(())
}

#[test]
fn protocol_bulk_update_arguments_use_named_levels() -> TestResult {
    for (level, name) in [
        (UpdateLevel::Latest, "latest"),
        (UpdateLevel::Major, "major"),
        (UpdateLevel::Minor, "minor"),
        (UpdateLevel::Patch, "patch"),
    ] {
        assert_round_trip(
            &BulkUpdateArguments {
                uri: document_uri()?,
                level,
            },
            &json!({"uri": DOCUMENT_URI, "level": name}),
        )?;
    }
    Ok(())
}

#[test]
fn protocol_update_dependency_arguments_round_trip_exact_json() -> TestResult {
    assert_round_trip(
        &UpdateDependencyArguments {
            uri: document_uri()?,
            name: "serde".to_owned(),
            requirement_range: requirement_range(),
            command: UpdateCommand::UpdateMajor,
            selected_version: Some("2.0.0".to_owned()),
        },
        &json!({
            "uri": DOCUMENT_URI,
            "name": "serde",
            "requirementRange": {
                "start": {"line": 3, "character": 12},
                "end": {"line": 3, "character": 18}
            },
            "command": "updateMajor",
            "selectedVersion": "2.0.0"
        }),
    )?;
    assert_round_trip(
        &UpdateDependencyArguments {
            uri: document_uri()?,
            name: "serde".to_owned(),
            requirement_range: requirement_range(),
            command: UpdateCommand::Update,
            selected_version: None,
        },
        &json!({
            "uri": DOCUMENT_URI,
            "name": "serde",
            "requirementRange": {
                "start": {"line": 3, "character": 12},
                "end": {"line": 3, "character": 18}
            },
            "command": "update"
        }),
    )?;
    Ok(())
}

#[test]
fn protocol_update_command_values_match_lens_command_names() -> TestResult {
    for (command, name) in [
        (UpdateCommand::Update, "update"),
        (UpdateCommand::UpdateMajor, "updateMajor"),
        (UpdateCommand::UpdateMinor, "updateMinor"),
        (UpdateCommand::UpdatePatch, "updatePatch"),
        (UpdateCommand::UpdateRelease, "updateRelease"),
        (UpdateCommand::UpdatePrerelease, "updatePrerelease"),
    ] {
        assert_eq!(serde_json::to_value(command)?, json!(name));
        assert_eq!(
            serde_json::from_value::<UpdateCommand>(json!(name))?,
            command
        );
    }
    Ok(())
}

#[test]
fn protocol_choose_build_arguments_round_trip_exact_json() -> TestResult {
    assert_round_trip(
        &ChooseBuildArguments {
            uri: document_uri()?,
            name: "serde".to_owned(),
            requirement_range: requirement_range(),
            requirement: "^1.0.0".to_owned(),
            builds: vec!["1.0.1".to_owned(), "1.0.2".to_owned()],
        },
        &json!({
            "uri": DOCUMENT_URI,
            "name": "serde",
            "requirementRange": {
                "start": {"line": 3, "character": 12},
                "end": {"line": 3, "character": 18}
            },
            "requirement": "^1.0.0",
            "builds": ["1.0.1", "1.0.2"]
        }),
    )
}

#[test]
fn protocol_document_arguments_round_trip_for_document_commands() -> TestResult {
    assert_round_trip(
        &DocumentArguments {
            uri: document_uri()?,
        },
        &json!({"uri": DOCUMENT_URI}),
    )
}
}
