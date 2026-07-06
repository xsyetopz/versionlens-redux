use toml_edit::{Key, Value as TomlValue};

use crate::model::Dependency;

mod inline;
mod spans;

use inline::{inline_dependency_value, inline_registry_name};
use spans::{CargoDependencySource, CargoDependencySpans, cargo_dependency_from_span};

pub(super) struct CargoTomlDependencyInput<'a> {
    pub(super) text: &'a str,
    pub(super) group: &'a str,
    pub(super) name: &'a str,
    pub(super) value: &'a TomlValue,
    pub(super) name_key: &'a Key,
    pub(super) value_key: &'a Key,
}

pub(super) fn toml_dependency(input: CargoTomlDependencyInput<'_>) -> Option<Dependency> {
    if let Some(requirement) = input.value.as_str() {
        return Some(cargo_dependency_from_span(
            CargoDependencySource {
                text: input.text,
                group: input.group,
                name: input.name,
                hosted_name: None,
                requirement,
                hosted_url: None,
            },
            CargoDependencySpans {
                name: input.name_key.span(),
                requirement: input.value.span(),
            },
        ));
    }

    if let Some(field_value) = workspace_dependency_value(input.value, input.value_key) {
        return Some(cargo_dependency_from_span(
            CargoDependencySource {
                text: input.text,
                group: input.group,
                name: input.name,
                hosted_name: None,
                requirement: "workspace:true",
                hosted_url: None,
            },
            CargoDependencySpans {
                name: input.name_key.span(),
                requirement: field_value.span(),
            },
        ));
    }

    let inline = input.value.as_inline_table()?;
    let package_name = inline
        .get("package")
        .and_then(|value| value.as_str())
        .unwrap_or(input.name);
    let field_value = inline_dependency_value(input.value)?;
    let registry_name = inline_registry_name(input.value);
    let hosted_url = if field_value.field == "path" {
        Some("path")
    } else {
        registry_name
    };
    Some(cargo_dependency_from_span(
        CargoDependencySource {
            text: input.text,
            group: input.group,
            name: input.name,
            hosted_name: (package_name != input.name).then_some(package_name),
            requirement: field_value.value.as_str()?,
            hosted_url,
        },
        CargoDependencySpans {
            name: input.name_key.span(),
            requirement: field_value.value.span(),
        },
    ))
}

fn workspace_dependency_value<'a>(value: &'a TomlValue, value_key: &Key) -> Option<&'a TomlValue> {
    if value.as_bool() == Some(true) && value_key.get() == "workspace" {
        return Some(value);
    }

    let workspace = value.as_inline_table()?.get("workspace")?;
    (workspace.as_bool() == Some(true)).then_some(workspace)
}
