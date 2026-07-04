use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode};

use crate::model::Dependency;

use super::hosted::{hosted_name, hosted_url};
use crate::pubspec_yaml::dependency::{
    scalar::scalar_dependency_from_source, source::PubspecDependencySource,
};

pub(super) fn version_mapping_dependency(
    source: &PubspecDependencySource<'_>,
    map: &MarkedMappingNode,
    version: &MarkedScalarNode,
) -> Option<Dependency> {
    let mut dependency = scalar_dependency_from_source(source, version)?;
    dependency.hosted_url = hosted_url(map);
    dependency.hosted_name = hosted_name(map);
    Some(dependency)
}
