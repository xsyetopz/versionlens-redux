pub(crate) mod properties;

use versionlens_model::Dependency;
use versionlens_model::ecosystem_config_namespace;

use crate::view::DependencyView;

pub(crate) fn into_dependency_views(dependencies: Vec<Dependency>) -> Vec<DependencyView> {
    dependencies.into_iter().map(dependency_view).collect()
}

pub(crate) fn dependency_view(dependency: Dependency) -> DependencyView {
    let ecosystem = ecosystem_config_namespace(dependency.ecosystem).to_owned();
    DependencyView {
        name: dependency.name,
        requirement: dependency.requirement,
        ecosystem,
        group: dependency.group,
        hosted_url: dependency.hosted_url,
        hosted_name: dependency.hosted_name,
        range: dependency.range,
        requirement_range: dependency.requirement_range,
    }
}
