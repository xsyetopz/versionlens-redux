use marked_yaml::types::{MarkedMappingNode, Node};

use crate::{
    model::Dependency,
    pnpm_yaml::{PnpmCollectContext, nodes::mapping_node, paths::PACKAGE_EXTENSION_GROUPS},
};

use super::mapping::collect_dependency_mapping;

pub(in crate::pnpm_yaml) fn collect_package_extensions(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    let Some(Node::Mapping(extensions)) = root.get_node("packageExtensions") else {
        return;
    };

    for (extension_key, extension) in extensions.iter() {
        let Some(extension) = mapping_node(extension) else {
            continue;
        };
        collect_package_extension(context, extension_key.as_str(), extension, out);
    }
}

fn collect_package_extension(
    context: &PnpmCollectContext<'_>,
    extension_key: &str,
    extension: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for dependency_group in PACKAGE_EXTENSION_GROUPS {
        let Some(Node::Mapping(dependencies)) = extension.get_node(dependency_group) else {
            continue;
        };
        let group = format!("packageExtensions.{extension_key}.{dependency_group}");
        collect_dependency_mapping(context, &group, dependencies, out);
    }
}
