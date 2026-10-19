use std::ops::Range;

use crate::manifest::adapter::line_span;
use crate::manifest::adapter::toml::TomlNode;
use crate::manifest::path_query::{PathNode, query};
use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Julia;

/// A `key = "value"` entry of a `Project.toml` table.
struct StringEntry {
    key: String,
    value: String,
    /// The whole line that holds the entry.
    line: Range<usize>,
    value_span: Range<usize>,
}

/// A `Project.toml` source: a `path`, or else the `url` of a repository.
struct SourceEntry {
    source: &'static str,
    value: String,
    line: Range<usize>,
    value_span: Range<usize>,
}

pub(crate) fn parse_julia_project(text: &str) -> Vec<Dependency> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let root = TomlNode::root(text, &document);

    let mut dependencies = vec![];
    if let Some(version) = project_version(text, root) {
        dependencies.push(version);
    }

    let deps = string_entries(text, root, "deps");
    let compat = string_entries(text, root, "compat");
    let sources = source_entries(text, root);
    let mut names = deps
        .iter()
        .map(|entry| entry.key.as_str())
        .collect::<Vec<_>>();
    for entry in &compat {
        if entry.key != "julia" && !names.contains(&entry.key.as_str()) {
            names.push(entry.key.as_str());
        }
    }

    for package in names {
        // A later entry of the same key wins, as in the file.
        if let Some(source) = sources.iter().rfind(|(key, _)| key == package) {
            dependencies.push(source_dependency(text, package, &source.1));
        } else if let Some(entry) = compat.iter().rfind(|entry| entry.key == package) {
            dependencies.push(string_entry_dependency(text, "compat", entry));
        } else if let Some(entry) = deps.iter().find(|entry| entry.key == package) {
            dependencies.push(Dependency {
                name: entry.key.clone(),
                requirement: "latest".to_owned(),
                ecosystem: Julia,
                group: "deps".to_owned(),
                hosted_url: None,
                hosted_name: Some(entry.value.clone()),
                range: offset_range(text, entry.line.start, entry.line.end),
                requirement_range: offset_range(text, entry.value_span.end, entry.value_span.end),
                requirement_prefix: " # compat ".to_owned(),
                requirement_suffix: "".to_owned(),
                canonical_reference: None,
            });
        }
    }

    if let Some(entry) = compat.iter().find(|entry| entry.key == "julia") {
        dependencies.push(string_entry_dependency(text, "compat", entry));
    }

    dependencies
}

/// The package named by `name`, at the `version` of the project.
fn project_version(text: &str, root: TomlNode<'_>) -> Option<Dependency> {
    let name = root.children_named("name").first()?.scalar()?;
    let version_node = *root.children_named("version").first()?;
    let version = version_node.scalar()?;
    let line = line_span(text, &(version_node.key_span()?.start..version.span.end));
    Some(Dependency {
        name: name.value,
        requirement: version.value,
        ecosystem: Julia,
        group: "version".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, line.start, line.end),
        requirement_range: offset_range(text, version.span.start, version.span.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

fn string_entries(text: &str, root: TomlNode<'_>, table: &str) -> Vec<StringEntry> {
    query(root, &[table, "*"])
        .into_iter()
        .filter_map(|found| {
            let value = found.node.scalar()?;
            let line = line_span(text, &(found.node.key_span()?.start..value.span.end));
            Some(StringEntry {
                key: found.keys[1].clone(),
                value: value.value,
                line,
                value_span: value.span,
            })
        })
        .collect()
}

fn source_entries(text: &str, root: TomlNode<'_>) -> Vec<(String, SourceEntry)> {
    query(root, &["sources", "*"])
        .into_iter()
        .filter_map(|found| {
            let (source, value) =
                [("path", "path"), ("git", "url")]
                    .into_iter()
                    .find_map(|(source, field)| {
                        Some((source, found.node.children_named(field).first()?.scalar()?))
                    })?;
            let line = line_span(text, &(found.node.key_span()?.start..value.span.end));
            Some((
                found.keys[1].clone(),
                SourceEntry {
                    source,
                    value: value.value,
                    line,
                    value_span: value.span,
                },
            ))
        })
        .collect()
}

fn string_entry_dependency(text: &str, group: &str, entry: &StringEntry) -> Dependency {
    Dependency {
        name: entry.key.clone(),
        requirement: entry.value.clone(),
        ecosystem: Julia,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, entry.line.start, entry.line.end),
        requirement_range: offset_range(text, entry.value_span.start, entry.value_span.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

fn source_dependency(text: &str, package: &str, source: &SourceEntry) -> Dependency {
    Dependency {
        name: package.to_owned(),
        requirement: source.value.clone(),
        ecosystem: Julia,
        group: "sources".to_owned(),
        hosted_url: Some(source.source.to_owned()),
        hosted_name: None,
        range: offset_range(text, source.line.start, source.line.end),
        requirement_range: offset_range(text, source.value_span.start, source.value_span.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

/// The `[[deps.Name]]` blocks of a `Manifest.toml`. A block runs to the next
/// `[[deps.` header, or to the end of the file.
pub(crate) fn parse_julia_manifest(text: &str) -> Vec<Dependency> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let root = TomlNode::root(text, &document);

    let mut blocks = query(root, &["deps", "*", "*"])
        .into_iter()
        .filter_map(|found| {
            let span = found.node.table_span()?;
            Some((line_span(text, &span).start, found))
        })
        .collect::<Vec<_>>();
    blocks.sort_by_key(|(start, _)| *start);

    let ends = blocks
        .iter()
        .skip(1)
        .map(|(start, _)| *start)
        .chain([text.len()])
        .collect::<Vec<_>>();

    blocks
        .iter()
        .zip(ends)
        .filter_map(|((start, found), end)| {
            let field = |name: &str| found.node.children_named(name).first()?.scalar();
            let source = field("path")
                .map(|entry| ("path", entry))
                .or_else(|| field("repo-url").map(|entry| ("git", entry)));
            let (hosted_url, requirement) = match source {
                Some((kind, entry)) => (Some(kind), entry),
                None => (None, field("version")?),
            };
            Some(Dependency {
                name: found.keys[1].clone(),
                requirement: requirement.value,
                ecosystem: Julia,
                group: "deps".to_owned(),
                hosted_url: hosted_url.map(str::to_owned),
                hosted_name: None,
                range: offset_range(text, *start, end),
                requirement_range: offset_range(text, requirement.span.start, requirement.span.end),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
                canonical_reference: None,
            })
        })
        .collect()
}
