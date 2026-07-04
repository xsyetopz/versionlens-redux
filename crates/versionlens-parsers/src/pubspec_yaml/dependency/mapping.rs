mod git;
mod hosted;
mod version;

use marked_yaml::types::MarkedMappingNode;

use crate::model::Dependency;

use super::scalar::scalar_dependency_from_source;
use super::source::PubspecDependencySource;
use git::git_value;
use version::version_mapping_dependency;

pub(super) fn mapping_dependency(
    source: &PubspecDependencySource<'_>,
    map: &MarkedMappingNode,
) -> Option<Dependency> {
    if let Some(version) = map.get_scalar("version") {
        return version_mapping_dependency(source, map, version);
    }

    if let Some(dependency) = map
        .get_scalar("path")
        .or_else(|| git_value(map))
        .and_then(|value| scalar_dependency_from_source(source, value))
    {
        return Some(dependency);
    }

    None
}
