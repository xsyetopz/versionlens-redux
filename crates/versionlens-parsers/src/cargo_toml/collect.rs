mod project;
mod table;

use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use project::collect_cargo_project_version;
use table::collect_cargo_table_dependency;

pub(super) struct CargoCollectContext<'a> {
    pub(super) text: &'a str,
    pub(super) dependency_paths: &'a [&'a str],
}

pub(super) fn collect_toml_value(
    context: &CargoCollectContext<'_>,
    keys: &[&Key],
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) {
    if keys.len() < 2 || collect_cargo_project_version(context, keys, value, out) {
        return;
    }

    if !matches!(keys.first().map(|key| key.get()), Some("package")) {
        collect_cargo_table_dependency(context, keys, value, out);
    }
}
