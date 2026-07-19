use versionlens_model::{Dependency, ecosystem_config_namespace};
use versionlens_vscode_model::DependencyPayload;

pub(crate) fn into_dependency_payloads(dependencies: Vec<Dependency>) -> Vec<DependencyPayload> {
    dependencies.into_iter().map(dependency_payload).collect()
}

pub fn dependency_payload(dependency: Dependency) -> DependencyPayload {
    let ecosystem = ecosystem_config_namespace(dependency.ecosystem).to_owned();
    DependencyPayload {
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
