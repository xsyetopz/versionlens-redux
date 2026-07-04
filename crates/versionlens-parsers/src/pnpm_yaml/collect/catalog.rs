use marked_yaml::types::{MarkedMappingNode, Node};

use crate::{
    model::Dependency,
    pnpm_yaml::{PnpmCollectContext, nodes::mapping_node},
};

use super::mapping::collect_dependency_mapping;

pub(in crate::pnpm_yaml) fn collect_catalog(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    group: &str,
    out: &mut Vec<Dependency>,
) {
    let Some(Node::Mapping(catalog)) = root.get_node(group) else {
        return;
    };

    collect_dependency_mapping(context, group, catalog, out);
}

pub(in crate::pnpm_yaml) fn collect_catalogs(
    context: &PnpmCollectContext<'_>,
    root: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    let Some(Node::Mapping(catalogs)) = root.get_node("catalogs") else {
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
