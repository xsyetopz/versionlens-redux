use crate::manifest::adapter::toml::collect_toml_dependencies;
use versionlens_model::Dependency;

mod collect;
mod dependency;
mod project_version;

use collect::{CargoCollectContext, collect_toml_value};

pub(crate) use project_version::OWN_VERSION_PATHS;

pub(crate) fn parse_cargo_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    collect_toml_dependencies(text, |text, keys, value, dependencies| {
        let context = CargoCollectContext {
            text,
            dependency_paths: &dependency_paths,
            own_version: dependency_paths == OWN_VERSION_PATHS,
        };
        collect_toml_value(&context, keys, value, dependencies);
    })
}

#[cfg(test)]
mod tests;
