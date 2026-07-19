use crate::toml_walk::walk_toml_values;
use versionlens_model::Dependency;

mod collect;
mod dependencies;
mod paths;

use collect::TomlKind::{Pipfile as TomlPipfile, Pyproject as TomlPyproject};
use collect::{TomlKind, TomlValueContext, collect_toml_value};
use paths::selected_dependency_paths;

type PythonTomlDependencies = Vec<Dependency>;

pub(crate) fn parse_pyproject_toml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> PythonTomlDependencies {
    parse_python_toml(text, TomlPyproject, dependency_paths)
}

pub(crate) fn parse_pipfile_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> PythonTomlDependencies {
    parse_python_toml(text, TomlPipfile, dependency_paths)
}

fn parse_python_toml(
    text: &str,
    kind: TomlKind,
    dependency_paths: &[&str],
) -> PythonTomlDependencies {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };

    let mut dependencies = vec![];
    let mut keys = vec![];
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
