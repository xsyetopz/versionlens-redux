use super::dependency::dependency;
use crate::path_patterns::path_or_member_enabled;
use crate::pnpm_yaml::PnpmCollectContext;
use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node;
use marked_yaml::types::Node::Mapping as YamlMapping;
use marked_yaml::types::Node::Scalar as YamlScalar;
use versionlens_model::Dependency;

type PnpmCatalogDependencies = Vec<Dependency>;

pub(in crate::pnpm_yaml) fn collect_catalog(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    group: &str,
    out: &mut PnpmCatalogDependencies,
) {
    let Some(YamlMapping(catalog)) = root.get_node(group) else {
        return;
    };

    collect_dependency_mapping(context, group, catalog, out);
}

pub(in crate::pnpm_yaml) fn collect_catalogs(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut PnpmCatalogDependencies,
) {
    let Some(YamlMapping(catalogs)) = root.get_node("catalogs") else {
        return;
    };

    for (catalog_key, catalog) in catalogs.iter() {
        let Some(catalog) = mapping_node(catalog) else {
            continue;
        };
        let group = format!("catalogs.{}", catalog_key.as_str());
        collect_dependency_mapping(context, &group, catalog, out);
    }
}

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

pub(super) fn collect_dependency_mapping(
    context: &PnpmCollectContext<'_>,
    group: &str,
    dependencies: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for (key, value) in dependencies.iter() {
        let YamlScalar(value) = value else {
            continue;
        };
        if !path_or_member_enabled(context.dependency_paths, group, Some(key.as_str())) {
            continue;
        }
        if let Some(dependency) = dependency(context.text, group, key, value) {
            out.push(dependency);
        }
    }
}

fn mapping_node(node: &Node) -> Option<&MarkedMappingNode> {
    let YamlMapping(mapping) = node else {
        return None;
    };
    Some(mapping)
}

const PACKAGE_EXTENSION_GROUPS: &[&str] = &[
    "dependencies",
    "devDependencies",
    "peerDependencies",
    "optionalDependencies",
];
