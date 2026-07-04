use toml_edit::Key;

use crate::model::Dependency;
use crate::path_patterns::path_or_member_enabled_exact;

use super::super::dependencies::{
    PythonDependencySource, PythonDependencySpans, dependency_from_span,
};
use super::super::paths::{TomlPathContext, is_poetry_dependency_path};
use super::{TomlValueContext, push_keyed_dependency};

pub(super) fn collect_poetry_table_dependency(
    context: &TomlValueContext<'_>,
    out: &mut Vec<Dependency>,
) -> bool {
    let Some((group, name_key)) = poetry_table_dependency(context.keys, context.dependency_paths)
    else {
        return false;
    };
    let Some(requirement) = context.value.as_str() else {
        return true;
    };

    out.push(dependency_from_span(
        PythonDependencySource {
            text: context.text,
            group: &group,
            name: name_key.get(),
            requirement,
            hosted_url: None,
        },
        PythonDependencySpans {
            name: name_key.span(),
            requirement: context.value.span(),
        },
    ));
    true
}

pub(super) fn collect_poetry_dependency(
    context: &TomlValueContext<'_>,
    paths: &TomlPathContext,
    out: &mut Vec<Dependency>,
) {
    if !is_poetry_dependency_path(context.keys)
        || !path_or_member_enabled_exact(
            context.dependency_paths,
            &paths.parent,
            context.keys.last().map(|key| key.get()),
        )
    {
        return;
    }

    push_keyed_dependency(context, &paths.parent, out);
}

fn poetry_table_dependency<'a>(
    keys: &'a [&'a Key],
    dependency_paths: &[&str],
) -> Option<(String, &'a Key)> {
    let field = keys.last()?.get();
    if !matches!(field, "version" | "path" | "git") {
        return None;
    }

    let name_key = *keys.get(keys.len().checked_sub(2)?)?;
    let group = keys[..keys.len() - 1]
        .iter()
        .map(|key| key.get())
        .collect::<Vec<_>>()
        .join(".");
    let parent_group = &keys[..keys.len() - 2]
        .iter()
        .map(|key| key.get())
        .collect::<Vec<_>>()
        .join(".");
    if path_or_member_enabled_exact(dependency_paths, parent_group, Some(name_key.get())) {
        Some((group, name_key))
    } else {
        None
    }
}
