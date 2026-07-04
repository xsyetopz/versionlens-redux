use marked_yaml::types::{MarkedMappingNode, Node};

use crate::{
    model::Dependency,
    path_patterns::path_or_member_enabled,
    pnpm_yaml::{PnpmCollectContext, dependency},
};

pub(in crate::pnpm_yaml::collect) fn collect_dependency_mapping(
    context: &PnpmCollectContext<'_>,
    group: &str,
    dependencies: &MarkedMappingNode,
    out: &mut Vec<Dependency>,
) {
    for (key, value) in dependencies.iter() {
        let Node::Scalar(value) = value else {
            continue;
        };
        if !path_or_member_enabled(context.dependency_paths, group, Some(key.as_str())) {
            continue;
        }
        if let Some(dependency) = dependency::dependency(context.text, group, key, value) {
            out.push(dependency);
        }
    }
}
