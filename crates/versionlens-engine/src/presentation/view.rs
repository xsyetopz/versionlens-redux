use serde::{Deserialize, Serialize};
use versionlens_model::{Range, TextEdit};

/// A code lens the engine wants shown. Editors map it to their own lens type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LensView {
    /// The range the lens is shown on.
    pub range: Range,
    /// The kind of the lens.
    pub kind: LensKind,
    /// The text of the lens.
    pub title: String,
    /// The command the lens runs, if any.
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
        /// The package to update.
        name: String,
        /// The selector of the requirement in the document.
        selector: String,
        /// How to choose the version.
        mode: Option<UpdateMode>,
        /// The version to update to.
        version: Option<String>,
    },
    /// Choose one of several builds of a version.
    ChooseBuild {
        /// The selector of the requirement in the document.
        selector: String,
        /// The package to update.
        name: String,
        /// The current requirement.
        requirement: String,
        /// The builds to choose from.
        builds: Vec<String>,
    },
    /// Open a file.
    OpenFile {
        /// The path of the file.
        path: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// How an update command chooses the version.
pub enum UpdateMode {
    /// The latest version the requirement allows.
    Update,
    /// The latest major version.
    UpdateMajor,
    /// The latest minor version.
    UpdateMinor,
    /// The latest patch version.
    UpdatePatch,
    /// The latest release version.
    UpdateRelease,
    /// The latest prerelease version.
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
    /// The name of the mode in the protocol.
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
    /// The title of the quick fix.
    pub title: String,
    /// The edit the quick fix applies.
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
/// How serious a diagnostic is.
pub enum Severity {
    /// An error.
    Error,
    /// A warning.
    Warning,
    /// An information message.
    Information,
    /// A hint.
    Hint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A dependency found in a document.
pub struct DependencyView {
    /// The package name.
    pub name: String,
    /// The requirement as written.
    pub requirement: String,
    /// The ecosystem name.
    pub ecosystem: String,
    /// The group of the manifest the dependency is in.
    pub group: String,
    /// The URL of the source when the dependency is not from a registry.
    pub hosted_url: Option<String>,
    /// The name in the source when the dependency is not from a registry.
    pub hosted_name: Option<String>,
    /// The range of the whole dependency.
    pub range: Range,
    /// The range of the requirement.
    pub requirement_range: Range,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A diagnostic on a range of a document.
pub struct DiagnosticView {
    /// The range the diagnostic covers.
    pub range: Range,
    /// The message.
    pub message: String,
    /// The severity.
    pub severity: Severity,
    /// The name of the source that reported it.
    pub source: Option<String>,
    /// The code, such as `update-available`.
    pub code: Option<String>,
    /// A URL with more about the code.
    pub code_description_url: Option<String>,
}

impl DiagnosticView {
    /// A diagnostic from the `VersionLens` source.
    pub(crate) fn new(
        range: Range,
        message: String,
        severity: Severity,
        code: String,
        code_description_url: Option<String>,
    ) -> Self {
        Self {
            range,
            message,
            severity,
            source: Some("VersionLens".to_owned()),
            code: Some(code),
            code_description_url,
        }
    }
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
mod legacy {
    //! Test-only flat view of a `LensView` action: its variant name and the positional
    //! string arguments of the golden `CodeLensPayload` shape. Tests and the golden capture
    //! compare against these.

    use super::{Action, LensView};

    impl LensView {
        /// The `Action` variant name, or `""` for a display-only lens.
        pub(crate) fn action_kind(&self) -> &'static str {
            match &self.action {
                None => "",
                Some(Action::UpdateDependency { .. }) => "UpdateDependency",
                Some(Action::ChooseBuild { .. }) => "ChooseBuild",
                Some(Action::OpenFile { .. }) => "OpenFile",
            }
        }

        pub(crate) fn arguments(&self) -> Vec<String> {
            match &self.action {
                None => Vec::new(),
                Some(Action::UpdateDependency {
                    name,
                    selector,
                    mode,
                    version,
                }) => {
                    let mut arguments = vec![name.clone(), selector.clone()];
                    arguments.extend(mode.map(|mode| mode.as_str().to_owned()));
                    arguments.extend(version.iter().cloned());
                    arguments
                }
                Some(Action::ChooseBuild {
                    selector,
                    name,
                    requirement,
                    builds,
                }) => {
                    let mut arguments = vec![selector.clone(), name.clone(), requirement.clone()];
                    arguments.extend(builds.iter().cloned());
                    arguments
                }
                Some(Action::OpenFile { path }) => vec![path.clone()],
            }
        }
    }
}
