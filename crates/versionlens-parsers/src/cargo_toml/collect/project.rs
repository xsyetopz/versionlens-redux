use toml_edit::{Key, Value as TomlValue};

use crate::model::Dependency;

use super::CargoCollectContext;
use crate::cargo_toml::{
    dependency::{CargoTomlDependencyInput, toml_dependency},
    paths::is_cargo_project_version,
};

pub(super) fn collect_cargo_project_version(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) -> bool {
    if !is_cargo_project_version(keys, context.dependency_paths) {
        return false;
    }

    if let Some(dependency) = toml_dependency(CargoTomlDependencyInput {
        text: context.text,
        group: "package",
        name: "version",
        value,
        name_key: keys[1],
    }) {
        out.push(dependency);
    }
    true
}
