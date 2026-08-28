use versionlens_model::Dependency;
use versionlens_providers::provider_id;

use crate::dependency::is_npm_package_manager;

pub(crate) fn dependency_signature(dependencies: &[Dependency]) -> String {
    let mut entries = dependencies
        .iter()
        .filter(|dependency| !is_ignored_dependency(dependency))
        .map(|dependency| {
            format!(
                "{}\0{}\0{}\0{}",
                provider_id(dependency.ecosystem),
                dependency.name,
                dependency.group,
                dependency.requirement
            )
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.join("\n")
}

fn is_ignored_dependency(dependency: &Dependency) -> bool {
    is_npm_package_manager(dependency) || is_ignored_requirement(&dependency.requirement)
}

fn is_ignored_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim();
    requirement.starts_with("catalog:") || requirement.starts_with("workspace:")
}

#[cfg(test)]
mod tests;
