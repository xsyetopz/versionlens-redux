use crate::positions::offset_range;
use crate::yaml::scalar_range;
use marked_yaml::parse_yaml;
use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::{Mapping as YamlMapping, Sequence as YamlSequence};
use versionlens_model::Ecosystem::Helm;

use versionlens_model::Dependency;

pub(crate) fn parse_helm_chart_yaml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    if !dependency_paths.is_empty() && !dependency_paths.contains(&"dependencies") {
        return vec![];
    }

    let Ok(root) = parse_yaml(0, text) else {
        return vec![];
    };
    let Some(root) = root.as_mapping() else {
        return vec![];
    };
    let Some(YamlSequence(entries)) = root.get_node("dependencies") else {
        return vec![];
    };

    entries
        .iter()
        .filter_map(|entry| {
            let YamlMapping(entry) = entry else {
                return None;
            };
            chart_dependency(text, entry)
        })
        .collect()
}

fn chart_dependency(text: &str, entry: &MarkedMappingNode) -> Option<Dependency> {
    let name = entry.get_scalar("name")?;
    let version = entry.get_scalar("version")?;
    let repository = entry.get_scalar("repository");
    let alias = entry.get_scalar("alias");

    let displayed_name = alias.unwrap_or(name);
    let name_range = scalar_range(text, displayed_name)?;
    let version_range = scalar_range(text, version)?;
    let (requirement, requirement_start, requirement_prefix) =
        helm_requirement_parts(version.as_str(), version_range.start);
    let hosted_url = helm_hosted_url(repository.map(|value| value.as_str()));
    let hosted_name = alias.map(|_| name.as_str().to_owned());

    Some(Dependency {
        name: displayed_name.as_str().to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Helm,
        group: "dependencies".to_owned(),
        hosted_url,
        hosted_name,
        range: offset_range(text, name_range.start, name_range.end),
        requirement_range: offset_range(
            text,
            requirement_start,
            version_range.start + version.as_str().len(),
        ),
        requirement_prefix,
        requirement_suffix: "".to_owned(),
    })
}

fn helm_requirement_parts(requirement: &str, value_start: usize) -> (&str, usize, String) {
    let trimmed = requirement.trim_start();
    let leading = requirement.len() - trimmed.len();
    for operator in ["~", "^", ">=", "<=", ">", "<", "="] {
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

fn helm_hosted_url(repository: Option<&str>) -> Option<String> {
    let repository = repository?.trim();
    if repository.is_empty() {
        return None;
    }
    if repository.starts_with("file:") {
        return Some("file".to_owned());
    }
    if repository.starts_with('@') || repository.starts_with("alias:") {
        return Some("repo-alias".to_owned());
    }
    Some(repository.to_owned())
}
