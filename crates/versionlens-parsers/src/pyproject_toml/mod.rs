use crate::manifest_support::toml_walk::collect_toml_dependencies;
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
    let dependency_paths = selected_dependency_paths(dependency_paths);
    collect_toml_dependencies(text, |text, keys, value, dependencies| {
        let context = TomlValueContext {
            text,
            keys,
            kind,
            dependency_paths: &dependency_paths,
            value,
        };
        collect_toml_value(&context, dependencies);
    })
}

#[cfg(test)]
mod tests;
