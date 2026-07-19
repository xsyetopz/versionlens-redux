use marked_yaml::types::MarkedScalarNode;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};
use versionlens_model::Ecosystem::Pub;
mod git;
mod hosted;
mod version;

use marked_yaml::types::MarkedMappingNode;

use crate::positions::offset_range;
use crate::yaml::{byte_offset, scalar_range};
use versionlens_model::Dependency;

use super::scalar::scalar_dependency_from_source;
use super::source::PubspecDependencySource;
use git::git_value;
use hosted::{hosted_name, hosted_url};
use version::version_mapping_dependency;

pub(super) fn mapping_dependency(
    source: &PubspecDependencySource<'_>,
    map: &MarkedMappingNode,
) -> Option<Dependency> {
    if let Some(mut dependency) = map
        .get_scalar("sdk")
        .and_then(|value| scalar_dependency_from_source(source, value))
    {
        dependency.requirement = format!("sdk:{}", dependency.requirement);
        return Some(dependency);
    }

    if let Some(dependency) = map
        .get_scalar("path")
        .or_else(|| git_value(map))
        .and_then(|value| scalar_dependency_from_source(source, value))
    {
        return Some(dependency);
    }

    if let Some(version) = map.get_scalar("version") {
        return version_mapping_dependency(source, map, version);
    }

    hosted_url(map).and_then(|url| hosted_dependency_without_version(source, map, url))
}

fn hosted_dependency_without_version(
    source: &PubspecDependencySource<'_>,
    map: &MarkedMappingNode,
    url: String,
) -> Option<Dependency> {
    let text = source.text;
    let key = source.key;
    let name_start = byte_offset(text, key.span().start()?.character())?;
    let insert_at = hosted_insert_offset(text, map)?;

    Some(Dependency {
        name: key.as_str().to_owned(),
        requirement: "".to_owned(),
        ecosystem: Pub,
        group: source.group.to_owned(),
        hosted_url: Some(url),
        hosted_name: hosted_name(map),
        range: offset_range(text, name_start, name_start + key.as_str().len()),
        requirement_range: offset_range(text, insert_at, insert_at),
        requirement_prefix: format!("\n{}version: ", child_indent(text, key)?),
        requirement_suffix: "".to_owned(),
    })
}

fn hosted_insert_offset(text: &str, map: &MarkedMappingNode) -> Option<usize> {
    let hosted = match map.get_node("hosted")? {
        YamlScalar(value) => value,
        YamlMapping(hosted) => hosted
            .get_scalar("url")
            .or_else(|| hosted.get_scalar("name"))?,
        YamlSequence(_) => return None,
    };
    let end = scalar_range(text, hosted)?.end;
    Some(
        text[end..]
            .find('\n')
            .map_or(text.len(), |offset| end + offset),
    )
}

fn child_indent(text: &str, key: &MarkedScalarNode) -> Option<String> {
    let key_start = byte_offset(text, key.span().start()?.character())?;
    let line_start = text[..key_start].rfind('\n').map_or(0, |index| index + 1);
    let indent = &text[line_start..key_start];
    Some(format!("{indent}  "))
}
