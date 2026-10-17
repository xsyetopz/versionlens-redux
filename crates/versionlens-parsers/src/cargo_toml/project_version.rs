use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use super::dependency::{CargoTomlDependencyInput, toml_dependency};

/// The version of a Cargo project: `[package] version`, its inherited form
/// `version.workspace = true`, or `[workspace.package] version`.
pub(super) fn cargo_project_version(
    text: &str,
    keys: &[&Key],
    value: &TomlValue,
) -> Option<Dependency> {
    let (name_key, value_key) = match keys {
        [package, version] if package.get() == "package" && version.get() == "version" => {
            (keys[1], keys[1])
        }
        [package, version, workspace]
            if package.get() == "package"
                && version.get() == "version"
                && workspace.get() == "workspace" =>
        {
            (keys[1], keys[2])
        }
        [workspace, package, version]
            if workspace.get() == "workspace"
                && package.get() == "package"
                && version.get() == "version" =>
        {
            (keys[2], keys[2])
        }
        _ => return None,
    };

    toml_dependency(CargoTomlDependencyInput {
        text,
        group: "package",
        name: "version",
        value,
        name_key,
        value_key,
    })
}
