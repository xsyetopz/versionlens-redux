use crate::pnpm_yaml::nodes::mapping_node;
use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::Mapping as YamlMapping;
use versionlens_model::Dependency;

use super::super::PnpmCollectContext;

use super::mapping::collect_dependency_mapping;

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
