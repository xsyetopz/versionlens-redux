use crate::manifest_support::toml_walk::collect_toml_dependencies;
use versionlens_model::Dependency;

mod collect;
mod dependency;
mod project_version;

use collect::{CargoCollectContext, collect_toml_value};
use project_version::cargo_project_version;

pub(crate) fn parse_cargo_toml_project_versions(text: &str) -> Vec<Dependency> {
    collect_toml_dependencies(text, |text, keys, value, dependencies| {
        dependencies.extend(cargo_project_version(text, keys, value));
    })
}

pub(crate) fn parse_cargo_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    collect_toml_dependencies(text, |text, keys, value, dependencies| {
        let context = CargoCollectContext {
            text,
            dependency_paths: &dependency_paths,
        };
        collect_toml_value(&context, keys, value, dependencies);
    })
}

#[cfg(test)]
mod tests;
