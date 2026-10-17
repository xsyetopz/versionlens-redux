//! The manifest table: one row per supported file type.

use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind};

mod default_paths;
pub(crate) mod path_query;
mod patterns;
mod table;

pub use table::MANIFESTS;

type ParsedDependencies = Vec<Dependency>;

/// The parser for a manifest kind: the document text, and the dependency
/// paths when the format selects dependencies by path. `WithPaths` holds the
/// default paths, used when the user configures none.
#[derive(Clone, Copy)]
pub(crate) enum ManifestParser {
    Direct(fn(&str) -> ParsedDependencies),
    WithPaths(
        fn(&str, &[&str]) -> ParsedDependencies,
        &'static [&'static str],
    ),
}

impl ManifestParser {
    pub(crate) fn parse(self, text: &str, paths: &[&str]) -> ParsedDependencies {
        match self {
            Self::Direct(parser) => parser(text),
            Self::WithPaths(parser, defaults) => {
                parser(text, if paths.is_empty() { defaults } else { paths })
            }
        }
    }
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
}

impl ManifestRow {
    /// The dependency paths used when the user configures none.
    pub fn default_dependency_paths(&self) -> &'static [&'static str] {
        match self.parser {
            ManifestParser::Direct(_) => &[],
            ManifestParser::WithPaths(_, defaults) => defaults,
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
