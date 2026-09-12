use crate::cargo_toml::paths::{CargoProjectVersionPath, cargo_project_version_path};
use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use super::CargoCollectContext;
use crate::cargo_toml::dependency::{CargoTomlDependencyInput, toml_dependency};

pub(super) fn collect_cargo_project_version(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) -> bool {
    let Some(path) = cargo_project_version_path(keys, context.dependency_paths) else {
        return false;
    };
    let (name_key, value_key) = match path {
        CargoProjectVersionPath::Package => (keys[1], keys[1]),
        CargoProjectVersionPath::PackageWorkspace => (keys[1], keys[2]),
        CargoProjectVersionPath::WorkspacePackage => (keys[2], keys[2]),
    };

    if let Some(dependency) = toml_dependency(CargoTomlDependencyInput {
        text: context.text,
        group: "package",
        name: "version",
        value,
        name_key,
        value_key,
    }) {
        out.push(dependency);
    }
    true
}
