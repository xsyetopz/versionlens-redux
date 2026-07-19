use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};

use crate::path_patterns::path_or_member_enabled;
use crate::positions::offset_range;
use crate::yaml::scalar_range;
use versionlens_model::Dependency;

use super::dependency::{dependency_from_node, scalar_dependency};
use super::paths::dependency_groups;
use versionlens_model::Ecosystem::Pub;

type PubspecDependencies = Vec<Dependency>;

pub(super) struct PubspecCollectContext<'a> {
    pub(super) text: &'a str,
    pub(super) dependency_paths: &'a [&'a str],
}

pub(super) fn collect_pubspec_version(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut PubspecDependencies,
) {
    if !context.dependency_paths.contains(&"version") {
        return;
    }

    if let Some((key, YamlScalar(value))) = root.iter().find(|(key, _)| key.as_str() == "version")
        && let Some(dependency) = scalar_dependency(context.text, "version", key, value)
    {
        out.push(dependency);
    }
}

pub(super) fn collect_pubspec_dependency_groups(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut PubspecDependencies,
) {
    for group in dependency_groups(context.dependency_paths) {
        collect_pubspec_dependency_group(context, root, group, out);
    }
}

pub(super) fn collect_pubspec_workspace(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut PubspecDependencies,
) {
    if !context.dependency_paths.contains(&"workspace") {
        return;
    }

    let Some(YamlSequence(entries)) = root.get_node("workspace") else {
        return;
    };

    for entry in entries.iter() {
        let YamlScalar(value) = entry else {
            continue;
        };
        if value.as_str().is_empty() {
            continue;
        }
        let Some(value_range) = scalar_range(context.text, value) else {
            continue;
        };
        out.push(Dependency {
            name: value.as_str().to_owned(),
            requirement: value.as_str().to_owned(),
            ecosystem: Pub,
            group: "workspace".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(context.text, value_range.start, value_range.end),
            requirement_range: offset_range(context.text, value_range.start, value_range.end),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }
}

fn collect_pubspec_dependency_group(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    group: &str,
    out: &mut PubspecDependencies,
) {
    let Some(YamlMapping(entries)) = root.get_node(group) else {
        return;
    };

    for (key, value) in entries.iter() {
        if !path_or_member_enabled(context.dependency_paths, group, Some(key.as_str())) {
            continue;
        }
        if let Some(dependency) = dependency_from_node(context.text, group, key, value) {
            out.push(dependency);
        }
    }
}
