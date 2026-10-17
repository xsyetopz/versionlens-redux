use crate::cargo_toml::dependency::{CargoTomlDependencyInput, toml_dependency};
use crate::manifest_support::toml_walk::selected_dependency_path;
use toml_edit::{Key, Value as TomlValue};
use versionlens_model::Dependency;

use super::CargoCollectContext;

pub(super) fn collect_cargo_table_dependency(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) {
    let Some(table_match) = selected_dependency_path(keys, context.dependency_paths) else {
        return;
    };
    let name_key = cargo_dependency_name_key(keys, table_match, context.dependency_paths);
    let group = cargo_dependency_group(keys);

    if let Some(dependency) = toml_dependency(CargoTomlDependencyInput {
        text: context.text,
        group: &group,
        name: name_key.get(),
        value,
        name_key,
        value_key: keys[keys.len() - 1],
    }) {
        out.push(dependency);
    }
}

/// A path ending in `.*` selects one dependency per table, as in
/// `[dependencies.serde]`, so the name is the table key. When the paths also
/// hold the path with one more `.*`, as `patch.*` and `patch.*.*` do, the
/// wildcard names a group of dependencies instead, and the entries directly
/// in it are named by their own keys.
fn cargo_dependency_name_key<'a>(
    keys: &'a [&'a Key],
    table_match: &str,
    dependency_paths: &[&str],
) -> &'a Key {
    if !table_match.ends_with(".*") || keys.len() < 3 {
        return keys[keys.len() - 1];
    }
    let groups_entries = dependency_paths.contains(&format!("{table_match}.*").as_str())
        && keys.len() == table_match.split('.').count() + 1;
    if groups_entries {
        keys[keys.len() - 1]
    } else {
        keys[keys.len() - 2]
    }
}

fn cargo_dependency_group(keys: &[&Key]) -> String {
    let group_end = if keys.last().is_some_and(|key| key.get() == "workspace") && keys.len() >= 3 {
        keys.len() - 2
    } else {
        keys.len() - 1
    };

    keys[..group_end]
        .iter()
        .map(|key| key.get())
        .collect::<Vec<_>>()
        .join(".")
}
