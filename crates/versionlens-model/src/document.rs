use serde::{Deserialize, Serialize};

use crate::{Ecosystem, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionableKind {
    Dependency,
    RuntimeConstraint,
    ProjectVersion,
    WorkspaceReference,
    EcosystemHandle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CanonicalReference {
    GitHubActionTag {
        tag: String,
    },
    GitHubActionSha {
        commit: String,
        tag: String,
        separator: String,
    },
    GitHubActionCommit {
        commit: String,
    },
    GitHubActionRef {
        reference: String,
    },
    GitHubActionLocal {
        path: String,
        reusable_workflow: bool,
    },
    GitHubActionExpression {
        expression: String,
    },
    GitHubActionInvalid {
        reference: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentInput {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub version: Option<u64>,
}

impl DocumentInput {
    pub fn new(
        uri: impl Into<String>,
        language_id: impl Into<String>,
        text: impl Into<String>,
        workspace_root: Option<String>,
    ) -> Self {
        Self {
            uri: uri.into(),
            language_id: language_id.into(),
            text: text.into(),
            workspace_root,
            version: None,
        }
    }

    #[must_use]
    pub fn with_version(self, version: u64) -> Self {
        Self {
            version: Some(version),
            ..self
        }
    }
}

pub fn registry_alias_requirement(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let Some(split) = spec.rfind('@').filter(|index| *index > 0) else {
        return Some("");
    };
    Some(&spec[split + 1..])
}

pub fn is_npm_dist_tag_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim();
    !requirement.is_empty()
        && requirement
            .chars()
            .any(|character| character.is_ascii_alphabetic())
        && requirement.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub name: String,
    pub requirement: String,
    pub ecosystem: Ecosystem,
    pub group: String,
    pub hosted_url: Option<String>,
    pub hosted_name: Option<String>,
    pub range: Range,
    pub requirement_range: Range,
    pub requirement_prefix: String,
    pub requirement_suffix: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canonical_reference: Option<CanonicalReference>,
}

impl Dependency {
    pub fn is_runtime_version(&self) -> bool {
        matches!(
            self.group.as_str(),
            "engines" | "rust-version" | "packageManager" | "devEngines.packageManager"
        ) || matches!(
            self.hosted_url.as_deref(),
            Some("toolchain" | "runtime-expression")
        )
    }

    pub fn is_runtime_constraint(&self) -> bool {
        matches!(self.group.as_str(), "engines" | "rust-version")
            || self.is_runtime_version()
                && (self.requirement.contains(['<', '>', '^', '~', '*', '|'])
                    || self
                        .requirement
                        .split('.')
                        .any(|part| matches!(part, "x" | "X")))
    }

    pub fn versionable_kind(&self) -> VersionableKind {
        if self.is_runtime_constraint() {
            return VersionableKind::RuntimeConstraint;
        }
        if self.ecosystem == Ecosystem::GitHub
            || self.group.starts_with("with.")
                && matches!(
                    self.hosted_url.as_deref(),
                    Some("toolchain" | "runtime-expression")
                )
        {
            return VersionableKind::EcosystemHandle;
        }
        if matches!(
            self.group.as_str(),
            "packageManager" | "devEngines.packageManager"
        ) {
            return VersionableKind::EcosystemHandle;
        }
        if self.requirement.starts_with("workspace:")
            || self.requirement.starts_with("catalog:")
            || self.requirement.starts_with("file:")
            || self.requirement.starts_with("link:")
            || self.requirement.starts_with("portal:")
            || matches!(
                self.hosted_url.as_deref(),
                Some("local" | "path" | "workspace")
            )
        {
            return VersionableKind::WorkspaceReference;
        }
        if self.is_project_version() {
            return VersionableKind::ProjectVersion;
        }
        VersionableKind::Dependency
    }

    fn is_project_version(&self) -> bool {
        match self.ecosystem {
            Ecosystem::Cargo => self.group == "package" && self.name == "version",
            Ecosystem::Maven => {
                self.group == "project.version" && self.name == "version"
                    || self.group == "version" && !self.name.is_empty()
            }
            Ecosystem::Dotnet => {
                self.group == "PropertyGroup"
                    && matches!(self.name.as_str(), "Version" | "AssemblyVersion")
            }
            Ecosystem::Python => self.group == "project" && self.name == "version",
            Ecosystem::Npm | Ecosystem::Composer => {
                self.group == "version" && self.name == self.requirement
            }
            Ecosystem::Deno => self.group == "version" && self.name.starts_with('@'),
            Ecosystem::Hex
            | Ecosystem::Hackage
            | Ecosystem::Julia
            | Ecosystem::Cran
            | Ecosystem::Opam => self.group == "version" && !self.name.is_empty(),
            Ecosystem::Pub => self.group == "version" && self.name == "version",
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests;
