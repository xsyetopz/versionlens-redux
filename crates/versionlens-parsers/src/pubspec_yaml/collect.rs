use marked_yaml::types::{MarkedMappingNode, Node};

use crate::model::Dependency;
use crate::path_patterns::path_or_member_enabled;

use super::dependency::{dependency_from_node, scalar_dependency};
use super::paths::dependency_groups;

pub(super) struct PubspecCollectContext<'a> {
    pub(super) text: &'a str,
    pub(super) dependency_paths: &'a [&'a str],
}

pub(super) fn collect_pubspec_version(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    if !context.dependency_paths.contains(&"version") {
        return;
    }

    if let Some((key, Node::Scalar(value))) = root.iter().find(|(key, _)| key.as_str() == "version")
        && let Some(dependency) = scalar_dependency(context.text, "version", key, value)
    {
        out.push(dependency);
    }
}

pub(super) fn collect_pubspec_dependency_groups(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for group in dependency_groups(context.dependency_paths) {
        collect_pubspec_dependency_group(context, root, group, out);
    }
}

fn collect_pubspec_dependency_group(
    context: &PubspecCollectContext<'_>,
    root: &MarkedMappingNode,
    group: &str,
    out: &mut Vec<Dependency>,
) {
    let Some(Node::Mapping(entries)) = root.get_node(group) else {
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
