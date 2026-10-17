use crate::manifest::path_query::selects;
use toml_edit::Item::{
    ArrayOfTables as TomlArrayOfTables, None as TomlNone, Table as TomlTable,
    Value as TomlItemValue,
};
use toml_edit::{Item, Key, Table, Value as TomlValue};
use versionlens_model::Dependency;

pub(crate) fn collect_toml_dependencies(
    text: &str,
    mut visit: impl FnMut(&str, &[&Key], &TomlValue, &mut Vec<Dependency>),
) -> Vec<Dependency> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };

    let mut dependencies = vec![];
    let mut keys = vec![];
    walk_toml_values(document.as_table(), &mut keys, &mut |keys, value| {
        visit(text, keys, value, &mut dependencies);
    });
    dependencies
}

pub(crate) fn walk_toml_values<'a>(
    table: &'a Table,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    for (key_name, item) in table.iter() {
        let Some(key) = table.key(key_name) else {
            continue;
        };

        keys.push(key);
        visit_toml_item(item, keys, visit);
        keys.pop();
    }
}

fn visit_toml_item<'a>(
    item: &'a Item,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    match item {
        TomlItemValue(value) => visit(keys, value),
        TomlTable(child) => walk_toml_values(child, keys, visit),
        TomlArrayOfTables(_) | TomlNone => {}
    }
}

/// The first of `dependency_paths` that selects the table holding the leaf at
/// `keys`. The leaf's own key is not part of its table path.
pub(crate) fn selected_dependency_path<'a>(
    keys: &[&Key],
    dependency_paths: &'a [&str],
) -> Option<&'a str> {
    let (_, table_keys) = keys.split_last()?;
    let table_keys = table_keys.iter().map(|key| key.get()).collect::<Vec<_>>();

    dependency_paths
        .iter()
        .copied()
        .find(|path| selects(&path.split('.').collect::<Vec<_>>(), &table_keys))
}
