use crate::positions::offset_range;
use crate::yaml::scalar_range;
use marked_yaml::types::Node;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};
use marked_yaml::{
    parse_yaml,
    types::{MarkedMappingNode, MarkedScalarNode},
};
use versionlens_model::Ecosystem::AnsibleGalaxy;

use versionlens_model::Dependency;

pub(crate) fn parse_ansible_galaxy_requirements_yaml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let Ok(root) = parse_yaml(0, text) else {
        return vec![];
    };

    let mut dependencies = vec![];
    match root {
        YamlMapping(root) => {
            if path_enabled(dependency_paths, "roles") {
                collect_group(text, &root, "roles", &mut dependencies);
            }
            if path_enabled(dependency_paths, "collections") {
                collect_group(text, &root, "collections", &mut dependencies);
            }
        }
        YamlSequence(entries) => {
            if path_enabled(dependency_paths, "roles") {
                collect_entries(text, "roles", entries.iter(), &mut dependencies);
            }
        }
        YamlScalar(_) => {}
    }

    dependencies
}

fn path_enabled(paths: &[&str], group: &str) -> bool {
    paths.is_empty() || paths.contains(&group)
}

fn collect_group(text: &str, root: &MarkedMappingNode, group: &str, out: &mut Vec<Dependency>) {
    let Some(YamlSequence(entries)) = root.get_node(group) else {
        return;
    };
    collect_entries(text, group, entries.iter(), out);
}

fn collect_entries<'a>(
    text: &str,
    group: &str,
    entries: impl Iterator<Item = &'a Node>,
    out: &mut Vec<Dependency>,
) {
    for entry in entries {
        match entry {
            YamlScalar(value) => {
                if let Some(dependency) = scalar_entry(text, group, value) {
                    out.push(dependency);
                }
            }
            YamlMapping(map) => {
                if map.get_scalar("include").is_some() {
                    continue;
                }
                if let Some(dependency) = mapping_entry(text, group, map) {
                    out.push(dependency);
                }
            }
            YamlSequence(_) => {}
        }
    }
}

fn scalar_entry(text: &str, group: &str, value: &MarkedScalarNode) -> Option<Dependency> {
    let name = value.as_str().trim();
    if name.is_empty() {
        return None;
    }
    let name_range = scalar_range(text, value)?;
    Some(Dependency {
        name: name.to_owned(),
        requirement: "latest".to_owned(),
        ecosystem: AnsibleGalaxy,
        group: group.to_owned(),
        hosted_url: (group == "roles").then(|| "role".to_owned()),
        hosted_name: None,
        range: offset_range(text, name_range.start, name_range.end),
        requirement_range: offset_range(text, name_range.start, name_range.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn mapping_entry(text: &str, group: &str, entry: &MarkedMappingNode) -> Option<Dependency> {
    let name = entry.get_scalar("name");
    let src = entry.get_scalar("src");
    let source = entry.get_scalar("source");
    let version = entry.get_scalar("version");

    let name_node = name.or(src)?;
    let dependency_name = dependency_name(
        group,
        name.map(|value| value.as_str()),
        src.map(|value| value.as_str()),
    );
    if dependency_name.is_empty() {
        return None;
    }

    let name_range = scalar_range(text, name_node)?;
    let (requirement, requirement_range, requirement_prefix) = version
        .and_then(|node| {
            let value_range = scalar_range(text, node)?;
            let (requirement, requirement_start, requirement_prefix) =
                ansible_requirement_parts(node.as_str(), value_range.start);
            Some((
                requirement.to_owned(),
                offset_range(
                    text,
                    requirement_start,
                    value_range.start + node.as_str().len(),
                ),
                requirement_prefix,
            ))
        })
        .unwrap_or_else(|| {
            (
                "latest".to_owned(),
                offset_range(text, name_range.start, name_range.end),
                "".to_owned(),
            )
        });

    Some(Dependency {
        name: dependency_name,
        requirement,
        ecosystem: AnsibleGalaxy,
        group: group.to_owned(),
        hosted_url: hosted_url(
            group,
            src.map(|value| value.as_str()),
            source.map(|value| value.as_str()),
        ),
        hosted_name: None,
        range: offset_range(text, name_range.start, name_range.end),
        requirement_range,
        requirement_prefix,
        requirement_suffix: "".to_owned(),
    })
}

fn dependency_name(group: &str, name: Option<&str>, src: Option<&str>) -> String {
    if let Some(name) = name
        .map(|value| value.trim())
        .filter(|name| !name.is_empty())
    {
        return name.to_owned();
    }

    let src = src.unwrap_or_default().trim();
    if group == "roles" && source_is_url_or_scm(src) {
        return source_basename(src);
    }
    src.to_owned()
}

fn hosted_url(group: &str, src: Option<&str>, source: Option<&str>) -> Option<String> {
    if let Some(src) = src.map(|value| value.trim()).filter(|src| !src.is_empty()) {
        if source_is_url_or_scm(src) {
            return Some("git".to_owned());
        }
    }

    if group == "roles" {
        return Some("role".to_owned());
    }

    source
        .map(|value| value.trim())
        .filter(|source| !source.is_empty())
        .map(|value| value.to_owned())
}

fn source_is_url_or_scm(source: &str) -> bool {
    source.contains("://") || source.starts_with("git@") || source.starts_with("git+")
}

fn source_basename(source: &str) -> String {
    let source = source
        .trim_end_matches('/')
        .rsplit(['/', ':'])
        .next()
        .unwrap_or(source)
        .trim_end_matches(".git");
    source.to_owned()
}

fn ansible_requirement_parts(requirement: &str, value_start: usize) -> (&str, usize, String) {
    let trimmed = requirement.trim_start();
    let leading = requirement.len() - trimmed.len();
    for operator in ["===", "!=", ">=", "<=", "==", ">", "<", "~", "^"] {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.is_empty() {
                break;
            }
            let whitespace = rest.len() - rest_trimmed.len();
            return (
                trimmed,
                value_start + leading + operator.len() + whitespace,
                operator.to_owned(),
            );
        }
    }
    (trimmed, value_start + leading, "".to_owned())
}
