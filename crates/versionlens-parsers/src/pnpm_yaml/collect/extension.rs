use crate::pnpm_yaml::nodes::mapping_node;
use crate::pnpm_yaml::paths::PACKAGE_EXTENSION_GROUPS;
use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::Mapping as YamlMapping;
use versionlens_model::Dependency;

use super::super::PnpmCollectContext;

use super::mapping::collect_dependency_mapping;

type PnpmExtensionDependencies = Vec<Dependency>;

pub(in crate::pnpm_yaml) fn collect_package_extensions(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut PnpmExtensionDependencies,
) {
    let Some(YamlMapping(extensions)) = root.get_node("packageExtensions") else {
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
    out: &mut PnpmExtensionDependencies,
) {
    for dependency_group in PACKAGE_EXTENSION_GROUPS {
        let Some(YamlMapping(dependencies)) = extension.get_node(dependency_group) else {
            continue;
        };
        let group = format!("packageExtensions.{extension_key}.{dependency_group}");
        collect_dependency_mapping(context, &group, dependencies, out);
    }
}
