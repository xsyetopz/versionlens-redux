use crate::manifest::adapter::line_span;
use crate::manifest::adapter::toml::TomlNode;
use crate::manifest::path_query::{PathNode, query};
use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Hex;

const DEPENDENCY_GROUPS: &[&str] = &["dependencies", "dev_dependencies", "dev-dependencies"];

pub(crate) fn parse_gleam_toml(text: &str) -> Vec<Dependency> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };
    let root = TomlNode::root(text, &document);

    let mut found = DEPENDENCY_GROUPS
        .iter()
        .flat_map(|group| query(root, &[group, "*"]))
        .filter_map(|entry| {
            let start = entry.node.key_span()?.start;
            let dependency = gleam_dependency(text, &entry.keys[0], &entry.keys[1], entry.node)?;
            Some((start, dependency))
        })
        .collect::<Vec<_>>();
    found.sort_by_key(|(start, _)| *start);

    let mut dependencies = found
        .into_iter()
        .map(|(_, dependency)| dependency)
        .collect::<Vec<_>>();
    if let Some(version) = project_version(text, root) {
        dependencies.insert(0, version);
    }
    dependencies
}

fn project_version(text: &str, root: TomlNode<'_>) -> Option<Dependency> {
    let name = root.children_named("name").first()?.scalar()?;
    let version_node = *root.children_named("version").first()?;
    let version = version_node.scalar()?;
    Some(Dependency {
        name: name.value,
        requirement: version.value,
        ecosystem: Hex,
        group: "version".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(text, version_node.key_span()?.start..version.span.end),
        requirement_range: offset_range(text, version.span.start, version.span.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

/// A string requirement, or the `path` or else the `git` of an inline table.
fn gleam_dependency(text: &str, group: &str, name: &str, node: TomlNode<'_>) -> Option<Dependency> {
    if !node.is_value() {
        return None;
    }
    let (requirement, hosted_url) = match node.scalar() {
        Some(requirement) => (requirement, None),
        None => ["path", "git"].into_iter().find_map(|field| {
            let value = node.children_named(field).first()?.scalar()?;
            Some((value, Some(field)))
        })?,
    };
    let start = node.key_span()?.start;
    let end = node.value_span()?.end;
    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.value,
        ecosystem: Hex,
        group: group.to_owned(),
        hosted_url: hosted_url.map(str::to_owned),
        hosted_name: None,
        range: line_range(text, start..end),
        requirement_range: offset_range(text, requirement.span.start, requirement.span.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

/// From `span.start` to the end of the line that holds `span.end`.
fn line_range(text: &str, span: std::ops::Range<usize>) -> versionlens_model::Range {
    let line = line_span(text, &span);
    offset_range(text, span.start.min(line.end), line.end)
}

#[cfg(test)]
mod tests;
