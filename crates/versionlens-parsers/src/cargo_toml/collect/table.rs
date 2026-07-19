use crate::cargo_toml::dependency::{CargoTomlDependencyInput, toml_dependency};
use crate::cargo_toml::paths::match_cargo_dependency_table;
use toml_edit::{Key, Value as TomlValue};
use versionlens_model::Dependency;

use super::CargoCollectContext;

pub(super) fn collect_cargo_table_dependency(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) {
    let Some(table_match) = match_cargo_dependency_table(keys, context.dependency_paths) else {
        return;
    };
    let name_key = cargo_dependency_name_key(keys, table_match);
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

fn cargo_dependency_name_key<'a>(keys: &'a [&'a Key], table_match: &str) -> &'a Key {
    if table_match.ends_with(".*") && keys.len() >= 3 {
        keys[keys.len() - 2]
    } else {
        keys[keys.len() - 1]
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
