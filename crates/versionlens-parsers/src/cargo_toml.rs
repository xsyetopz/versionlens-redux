use crate::toml_walk::walk_toml_values;
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
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };

    let mut dependencies = vec![];
    let mut keys = vec![];
    let dependency_paths = selected_dependency_paths(dependency_paths);
    let context = CargoCollectContext {
        text,
        dependency_paths: &dependency_paths,
    };
    walk_toml_values(document.as_table(), &mut keys, &mut |keys, value| {
        collect_toml_value(&context, keys, value, &mut dependencies);
    });
    dependencies
}

#[cfg(test)]
mod tests;
