use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::Scalar as YamlScalar;

use super::super::dependency::dependency;
use crate::path_patterns::path_or_member_enabled;
use crate::pnpm_yaml::PnpmCollectContext;
use versionlens_model::Dependency;

pub(in crate::pnpm_yaml::collect) fn collect_dependency_mapping(
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
