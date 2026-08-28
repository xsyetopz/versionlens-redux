use crate::manifest_support::toml_walk::collect_toml_dependencies;
use versionlens_model::Dependency;

mod collect;
mod dependency;
mod paths;

use collect::{CargoCollectContext, collect_toml_value};
use paths::selected_dependency_paths;

pub(crate) fn parse_cargo_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let dependency_paths = selected_dependency_paths(dependency_paths);
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
