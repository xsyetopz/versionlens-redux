use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use super::collect::CargoCollectContext;
use super::dependency::{CargoTomlDependencyInput, toml_dependency};
use crate::manifest::path_query::selects;

/// The version of a Cargo project, when the configured paths select it: the
/// leaf at `keys` is named by a path. The dependency is named `version`; the
/// key that holds the value is the leaf, and the name is the `version` key
/// above it, which is the same key unless the value is `version.workspace`.
pub(super) fn cargo_project_version(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
) -> Option<Dependency> {
    let names = keys.iter().map(|key| key.get()).collect::<Vec<_>>();
    if !context.dependency_paths.iter().any(|path| {
        let segments = path.split('.').collect::<Vec<_>>();
        segments.len() == names.len() && selects(&segments, &names)
    }) {
        return None;
    }
    let name_key = keys.iter().rev().find(|key| key.get() == "version")?;

    toml_dependency(CargoTomlDependencyInput {
        text: context.text,
        group: "package",
        name: "version",
        value,
        name_key,
        value_key: keys.last()?,
    })
}
