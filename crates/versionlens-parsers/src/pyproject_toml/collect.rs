use toml_edit::{Key, Value as TomlValue};

use crate::model::Dependency;

use super::dependencies::{PythonKeyedDependencyInput, keyed_dependency};
use super::paths::paths_for_keys;

mod arrays;
mod pipfile;
mod poetry;
mod project;
mod uv;

use arrays::collect_array_dependency_group;
use pipfile::collect_pipfile_dependency;
use poetry::{collect_poetry_dependency, collect_poetry_table_dependency};
use project::collect_project_version;
use uv::collect_uv_source;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TomlKind {
    Pipfile,
    Pyproject,
}

pub(super) struct TomlValueContext<'a> {
    pub(super) text: &'a str,
    pub(super) keys: &'a [&'a Key],
    pub(super) kind: TomlKind,
    pub(super) dependency_paths: &'a [&'a str],
    pub(super) value: &'a TomlValue,
}

pub(super) fn collect_toml_value(context: &TomlValueContext<'_>, out: &mut Vec<Dependency>) {
    let paths = paths_for_keys(context.keys);

    if collect_pipfile_dependency(context, &paths, out) {
        return;
    }

    if collect_project_version(context, &paths, out) {
        return;
    }

    if collect_array_dependency_group(context, &paths, out) {
        return;
    }

    if collect_uv_source(context, &paths, out) {
        return;
    }

    if collect_poetry_table_dependency(context, out) {
        return;
    }

    collect_poetry_dependency(context, &paths, out);
}

pub(in crate::pyproject_toml::collect) fn push_keyed_dependency(
    context: &TomlValueContext<'_>,
    group: &str,
    out: &mut Vec<Dependency>,
) {
    let Some(name_key) = context.keys.last() else {
        return;
    };
    if let Some(dependency) = keyed_dependency(PythonKeyedDependencyInput {
        text: context.text,
        group,
        name: name_key.get(),
        value: context.value,
        key: name_key,
    }) {
        out.push(dependency);
    }
}
