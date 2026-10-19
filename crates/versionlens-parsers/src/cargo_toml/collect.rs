mod table;

use super::project_version::{cargo_project_version, is_own_version_leaf};

use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use super::dependency::{CargoTomlDependencyInput, toml_dependency};
use table::collect_cargo_table_dependency;

pub(super) struct CargoCollectContext<'a> {
    pub(super) text: &'a str,
    pub(super) dependency_paths: &'a [&'a str],
    /// Whether the walk reads the version fields, and nothing else.
    pub(super) own_version: bool,
}

pub(super) fn collect_toml_value(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) {
    if keys.len() < 2 {
        return;
    }

    if context.own_version {
        out.extend(cargo_project_version(context, keys, value));
        return;
    }

    if is_own_version_leaf(keys) {
        return;
    }

    if keys.len() == 2
        && keys[0].get() == "package"
        && keys[1].get() == "rust-version"
        && context.dependency_paths.contains(&"package")
        && value.as_str().is_some()
    {
        if let Some(mut dependency) = toml_dependency(CargoTomlDependencyInput {
            text: context.text,
            group: "rust-version",
            name: "rust",
            value,
            name_key: keys[1],
            value_key: keys[1],
        }) {
            dependency.hosted_url = Some("toolchain".to_owned());
            out.push(dependency);
        }
        return;
    }

    if !matches!(keys.first().map(|key| key.get()), Some("package")) {
        collect_cargo_table_dependency(context, keys, value, out);
    }
}
