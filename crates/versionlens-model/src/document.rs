use serde::{Deserialize, Serialize};

use crate::{Ecosystem, Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// What a dependency entry's version text refers to, which decides how it may be updated.
pub enum VersionableKind {
    /// A package dependency requirement.
    Dependency,
    /// A constraint on the runtime or toolchain, such as `engines` or `rust-version`.
    RuntimeConstraint,
    /// The project's own version.
    ProjectVersion,
    /// A reference to a local or workspace package that has no registry version.
    WorkspaceReference,
    /// A handle resolved by the ecosystem itself, such as a GitHub Action or package manager pin.
    EcosystemHandle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// A parsed GitHub Actions `uses` reference, classified by how it pins its target.
pub enum CanonicalReference {
    /// A reference pinned to a tag.
    GitHubActionTag {
        /// Tag the reference is pinned to.
        tag: String,
    },
    /// A reference pinned to a commit SHA with a trailing tag comment.
    GitHubActionSha {
        /// Commit SHA the reference is pinned to.
        commit: String,
        /// Tag named in the trailing comment.
        tag: String,
        /// Text between the commit SHA and the tag comment.
        separator: String,
    },
    /// A reference pinned to a commit SHA without a tag.
    GitHubActionCommit {
        /// Commit SHA the reference is pinned to.
        commit: String,
    },
    /// A reference pinned to a non-tag ref such as a branch.
    GitHubActionRef {
        /// The reference text as written.
        reference: String,
    },
    /// A reference to an action or workflow in the same repository.
    GitHubActionLocal {
        /// Path of the local action or workflow.
        path: String,
        /// Whether the path names a reusable workflow rather than an action.
        reusable_workflow: bool,
    },
    /// A reference built from a `${{ }}` expression.
    GitHubActionExpression {
        /// The expression text as written.
        expression: String,
    },
    /// A reference that could not be parsed.
    GitHubActionInvalid {
        /// The reference text as written.
        reference: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A document submitted for dependency analysis.
pub struct DocumentInput {
    /// URI of the document.
    pub uri: String,
    /// Editor language identifier of the document.
    pub language_id: String,
    /// Full text of the document.
    pub text: String,
    /// Root of the workspace that contains the document, if any.
    pub workspace_root: Option<String>,
    #[serde(default)]
    /// Editor-supplied document version, if any.
    pub version: Option<u64>,
}

impl DocumentInput {
    /// Create an input with no document version.
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
    /// Return this input with the document version set to `version`.
    pub fn with_version(self, version: u64) -> Self {
        Self {
            version: Some(version),
            ..self
        }
    }
}

/// For a `jsr:` or `npm:` alias requirement, return the version part after the package name (empty if absent); `None` for other requirements.
pub fn registry_alias_requirement(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let Some(split) = spec.rfind('@').filter(|index| *index > 0) else {
        return Some("");
    };
    Some(&spec[split + 1..])
}

/// Report whether `requirement` looks like an npm dist-tag such as `latest` or `next`.
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
/// One versioned dependency entry found in a manifest.
pub struct Dependency {
    /// Package name.
    pub name: String,
    /// Version requirement text without its prefix and suffix.
    pub requirement: String,
    /// Ecosystem the dependency belongs to.
    pub ecosystem: Ecosystem,
    /// Manifest section the dependency was declared in, such as `dependencies`.
    pub group: String,
    /// Registry or source URL, or a marker such as `toolchain`, `local`, or `workspace`.
    pub hosted_url: Option<String>,
    /// Package name on the hosting registry when it differs from `name`.
    pub hosted_name: Option<String>,
    /// Document range of the whole dependency entry.
    pub range: Range,
    /// Document range of the requirement text.
    pub requirement_range: Range,
    /// Text immediately before the requirement that an edit must keep.
    pub requirement_prefix: String,
    /// Text immediately after the requirement that an edit must keep.
    pub requirement_suffix: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Parsed GitHub Actions reference, when the dependency is one.
    pub canonical_reference: Option<CanonicalReference>,
}

impl Dependency {
    /// Report whether this entry versions a runtime or toolchain rather than a package.
    pub fn is_runtime_version(&self) -> bool {
        matches!(
            self.group.as_str(),
            "engines" | "rust-version" | "packageManager" | "devEngines.packageManager"
        ) || matches!(
            self.hosted_url.as_deref(),
            Some("toolchain" | "runtime-expression")
        )
    }

    /// Report whether this entry constrains the runtime to a range rather than pinning one version.
    pub fn is_runtime_constraint(&self) -> bool {
        matches!(self.group.as_str(), "engines" | "rust-version")
            || self.is_runtime_version()
                && (self.requirement.contains(['<', '>', '^', '~', '*', '|'])
                    || self
                        .requirement
                        .split('.')
                        .any(|part| matches!(part, "x" | "X")))
    }

    /// Whether this is a Cargo dependency requirement, which the Cargo dialect reads
    /// (a bare version is a caret requirement). A Cargo project version and a Rust
    /// toolchain version (`rust-toolchain`, `channel`) stay literal.
    pub fn is_cargo_dependency_requirement(&self) -> bool {
        self.ecosystem == Ecosystem::Cargo
            && !self.is_runtime_version()
            && matches!(self.versionable_kind(), VersionableKind::Dependency)
    }

    /// Classify what this entry's version text refers to.
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
mod tests {
    use super::{Dependency, VersionableKind};
    use crate::{Ecosystem, Position, Range};

    fn dependency(ecosystem: Ecosystem, group: &str, name: &str, requirement: &str) -> Dependency {
        let range = Range {
            start: Position {
                line: 0,
                character: 0,
            },
            end: Position {
                line: 0,
                character: 1,
            },
        };
        Dependency {
            name: name.to_owned(),
            requirement: requirement.to_owned(),
            ecosystem,
            group: group.to_owned(),
            hosted_url: None,
            hosted_name: None,
            range,
            requirement_range: range,
            requirement_prefix: String::new(),
            requirement_suffix: String::new(),
            canonical_reference: None,
        }
    }

    #[test]
    fn version_category_preserves_exact_project_predicates() {
        assert_eq!(
            dependency(Ecosystem::Maven, "version", "demo", "0.1.0").versionable_kind(),
            VersionableKind::ProjectVersion
        );
        assert_eq!(
            dependency(Ecosystem::Maven, "dependencies", "demo", "0.1.0").versionable_kind(),
            VersionableKind::Dependency
        );
        assert_eq!(
            dependency(Ecosystem::Deno, "version", "plain", "1.0.0").versionable_kind(),
            VersionableKind::Dependency
        );
        assert_eq!(
            dependency(Ecosystem::Deno, "version", "@scope/pkg", "1.0.0").versionable_kind(),
            VersionableKind::ProjectVersion
        );
        assert_eq!(
            dependency(Ecosystem::Pub, "version", "other", "1.0.0").versionable_kind(),
            VersionableKind::Dependency
        );
        assert_eq!(
            dependency(Ecosystem::Pub, "version", "version", "1.0.0").versionable_kind(),
            VersionableKind::ProjectVersion
        );
    }

    #[test]
    fn local_package_protocols_are_workspace_references() {
        for requirement in [
            "file:../foo",
            "link:../foo",
            "portal:../foo",
            "workspace:*",
            "catalog:shared",
        ] {
            assert_eq!(
                dependency(Ecosystem::Npm, "dependencies", "foo", requirement).versionable_kind(),
                VersionableKind::WorkspaceReference,
            );
        }
        assert_eq!(
            dependency(
                Ecosystem::Npm,
                "dependencies",
                "foo",
                "https://example.test/foo.tgz"
            )
            .versionable_kind(),
            VersionableKind::Dependency,
        );
    }
}
