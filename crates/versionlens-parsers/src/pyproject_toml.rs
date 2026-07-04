use toml_edit::Document;

use crate::model::Dependency;
use crate::toml_walk::walk_toml_values;

mod collect;
mod dependencies;
mod paths;

use collect::{TomlKind, TomlValueContext, collect_toml_value};
use paths::selected_dependency_paths;

pub(crate) fn parse_pyproject_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    parse_python_toml(text, TomlKind::Pyproject, dependency_paths)
}

pub(crate) fn parse_pipfile_with_paths(text: &str, dependency_paths: &[&str]) -> Vec<Dependency> {
    parse_python_toml(text, TomlKind::Pipfile, dependency_paths)
}

fn parse_python_toml(text: &str, kind: TomlKind, dependency_paths: &[&str]) -> Vec<Dependency> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };

    let mut dependencies = Vec::new();
    let mut keys = Vec::new();
    let dependency_paths = selected_dependency_paths(dependency_paths);
    walk_toml_values(document.as_table(), &mut keys, &mut |keys, value| {
        let context = TomlValueContext {
            text,
            keys,
            kind,
            dependency_paths: &dependency_paths,
            value,
        };
        collect_toml_value(&context, &mut dependencies);
    });
    dependencies
}

#[cfg(test)]
mod tests;
