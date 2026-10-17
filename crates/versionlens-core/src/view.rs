use serde::{Deserialize, Serialize};
use versionlens_model::{Range, TextEdit};

/// A code lens the engine wants shown. Editors map it to their own lens type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LensView {
    pub range: Range,
    pub kind: LensKind,
    pub title: String,
    pub action: Option<Action>,
    /// Position among the lenses of one dependency, from 0. Editors show
    /// lenses that share a range in this order.
    pub order: u32,
}

impl LensView {
    pub(crate) fn new(range: Range, title: String, action: Option<Action>) -> Self {
        let kind = if action.is_some() {
            LensKind::Command
        } else {
            LensKind::Display
        };
        Self {
            range,
            kind,
            title,
            action,
            order: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LensKind {
    /// Text only. The lens has no action.
    Display,
    /// The lens carries an `Action`.
    Command,
}

/// A typed command. Editors map each variant to their own command IDs and arguments.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    /// `mode` and `version` are set together for an update choice, `mode` alone for a
    /// project-version lens, and both are `None` for a plain update.
    UpdateDependency {
        name: String,
        selector: String,
        mode: Option<UpdateMode>,
        version: Option<String>,
    },
    ChooseBuild {
        selector: String,
        name: String,
        requirement: String,
        builds: Vec<String>,
    },
    OpenFile {
        path: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateMode {
    Update,
    UpdateMajor,
    UpdateMinor,
    UpdatePatch,
    UpdateRelease,
    UpdatePrerelease,
}

impl UpdateMode {
    const ALL: [Self; 6] = [
        Self::Update,
        Self::UpdateMajor,
        Self::UpdateMinor,
        Self::UpdatePatch,
        Self::UpdateRelease,
        Self::UpdatePrerelease,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Update => "update",
            Self::UpdateMajor => "updateMajor",
            Self::UpdateMinor => "updateMinor",
            Self::UpdatePatch => "updatePatch",
            Self::UpdateRelease => "updateRelease",
            Self::UpdatePrerelease => "updatePrerelease",
        }
    }

    pub(crate) fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.as_str() == value)
    }
}

/// A quick fix that bumps one requirement to an offered version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpActionView {
    pub title: String,
    pub edit: TextEdit,
    /// Whether the bump is to the latest release.
    pub preferred: bool,
    /// The update diagnostic of the requirement, when it has one.
    pub diagnostic: Option<DiagnosticView>,
}

/// The code of the diagnostic on a requirement that does not allow the
/// latest release.
pub const UPDATE_AVAILABLE: &str = "update-available";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyView {
    pub name: String,
    pub requirement: String,
    pub ecosystem: String,
    pub group: String,
    pub hosted_url: Option<String>,
    pub hosted_name: Option<String>,
    pub range: Range,
    pub requirement_range: Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticView {
    pub range: Range,
    pub message: String,
    pub severity: Severity,
    pub source: Option<String>,
    pub code: Option<String>,
    pub code_description_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusView {
    pub dependency_count: u32,
    pub update_count: u32,
    pub vulnerability_count: u32,
    pub error_count: u32,
    pub no_match_count: u32,
    pub visible: bool,
    pub text: String,
    pub tooltip: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionView {
    pub dependency: DependencyView,
    pub latest: Option<String>,
    pub status: String,
    pub builds: Vec<String>,
}

#[cfg(test)]
mod legacy;
