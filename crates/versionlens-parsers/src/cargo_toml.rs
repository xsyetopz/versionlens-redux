use toml_edit::Document;

use crate::model::Dependency;
use crate::toml_walk::walk_toml_values;

mod collect;
mod dependency;
mod paths;

use collect::{CargoCollectContext, collect_toml_value};
use paths::selected_dependency_paths;

pub(crate) fn parse_cargo_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    let mut keys = Vec::new();
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
