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
}

impl Dependency {
    pub fn versionable_kind(&self) -> VersionableKind {
        if self.ecosystem == Ecosystem::GitHub {
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
        if matches!(self.group.as_str(), "engines" | "rust-version") {
            return VersionableKind::RuntimeConstraint;
        }
        VersionableKind::Dependency
    }

    fn is_project_version(&self) -> bool {
        match self.ecosystem {
            Ecosystem::Cargo => self.group == "package" && self.name == "version",
            Ecosystem::Maven => self.group == "project.version" && self.name == "version",
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
