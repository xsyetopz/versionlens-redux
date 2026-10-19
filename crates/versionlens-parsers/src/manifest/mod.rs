//! The manifest table: one row per supported file type.

use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind};

pub(crate) mod adapter;
pub(crate) mod path_query;
mod patterns;
mod table;

pub use table::MANIFESTS;

type ParsedDependencies = Vec<Dependency>;

/// The parser for a manifest kind: the document text, and the dependency
/// path queries when the format selects dependencies by path.
#[derive(Clone, Copy)]
pub(crate) enum ManifestParser {
    Direct(fn(&str) -> ParsedDependencies),
    WithPaths(fn(&str, &[&str]) -> ParsedDependencies),
}

/// How a manifest file is written. `Custom` file types keep a hand-written parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestFormat {
    Json,
    Toml,
    Yaml,
    Xml,
    Custom,
}

/// The order in which classification tries patterns. A document gets the kind
/// of the first row that matches in the earliest stage, so a later stage only
/// sees documents that no earlier stage claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    Early,
    Editor,
    Workflow,
    Late,
    Content,
    Python,
    Pubspec,
}

impl Stage {
    pub const ALL: [Stage; 7] = [
        Stage::Early,
        Stage::Editor,
        Stage::Workflow,
        Stage::Late,
        Stage::Content,
        Stage::Python,
        Stage::Pubspec,
    ];
}

#[derive(Clone, Copy)]
pub enum Rule {
    /// The file name, compared without ASCII case.
    Name(&'static str),
    /// The file extension, compared without ASCII case.
    Extension(&'static str),
    /// The editor language ID.
    Language(&'static str),
    /// A test on the whole document URI.
    Uri(fn(&str) -> bool),
    /// A test on the document text.
    Content(fn(&DocumentInput, &str) -> bool),
}

#[derive(Clone, Copy)]
pub struct Pattern {
    pub stage: Stage,
    pub rule: Rule,
}

pub struct ManifestRow {
    pub kind: ManifestKind,
    pub ecosystem: Ecosystem,
    /// Provider family name, such as `golang` or `pypi`.
    pub provider: &'static str,
    pub format: ManifestFormat,
    pub patterns: &'static [Pattern],
    pub(crate) parser: ManifestParser,
    /// The path queries that select dependencies, used when the user
    /// configures none. Empty for file types that have no paths.
    pub dependency_paths: &'static [&'static str],
    /// Path queries for the file's own version fields, such as the `version`
    /// of a `package.json`. Dependency parsing leaves them out, even when the
    /// user configures them, and `parse_own_version` reads them.
    pub(crate) own_version: &'static [&'static str],
}

impl ManifestRow {
    pub(crate) const fn dependency_paths(mut self, paths: &'static [&'static str]) -> Self {
        self.dependency_paths = paths;
        self
    }

    pub(crate) const fn own_version(mut self, paths: &'static [&'static str]) -> Self {
        self.own_version = paths;
        self
    }

    /// The dependencies of `text`, without the file's own version fields.
    pub(crate) fn parse(&self, text: &str, paths: &[&str]) -> ParsedDependencies {
        let paths = paths
            .iter()
            .copied()
            .filter(|path| !self.own_version.contains(path))
            .collect::<Vec<_>>();
        match self.parser {
            ManifestParser::Direct(parser) => parser(text),
            ManifestParser::WithPaths(parser) => parser(
                text,
                if paths.is_empty() {
                    self.dependency_paths
                } else {
                    &paths
                },
            ),
        }
    }

    /// The file's own version fields, which only project-version commands edit.
    pub(crate) fn parse_own_version(&self, text: &str) -> ParsedDependencies {
        if self.own_version.is_empty() {
            return Vec::new();
        }
        match self.parser {
            ManifestParser::Direct(_) => Vec::new(),
            ManifestParser::WithPaths(parser) => parser(text, self.own_version),
        }
    }
}

impl Rule {
    pub(crate) fn matches(&self, input: &DocumentInput, uri: &str) -> bool {
        let name = patterns::file_name(uri);
        match *self {
            Rule::Name(expected) => name.is_some_and(|name| name.eq_ignore_ascii_case(expected)),
            Rule::Extension(expected) => name.is_some_and(|name| {
                crate::path(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
            }),
            Rule::Language(expected) => input.language_id == expected,
            Rule::Uri(test) => test(uri),
            Rule::Content(test) => test(input, uri),
        }
    }
}

pub fn manifest_row(kind: ManifestKind) -> Option<&'static ManifestRow> {
    MANIFESTS.iter().find(|row| row.kind == kind)
}

pub fn ecosystem_for_manifest(kind: ManifestKind) -> Option<Ecosystem> {
    manifest_row(kind).map(|row| row.ecosystem)
}

pub fn provider_name_for_manifest(kind: ManifestKind) -> Option<&'static str> {
    manifest_row(kind).map(|row| row.provider)
}

#[cfg(test)]
mod tests;
