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
}

pub(super) fn toml_dependency(input: CargoTomlDependencyInput<'_>) -> Option<Dependency> {
    if let Some(requirement) = input.value.as_str() {
        return Some(cargo_dependency_from_span(
            CargoDependencySource {
                text: input.text,
                group: input.group,
                name: input.name,
                requirement,
                hosted_url: None,
            },
            CargoDependencySpans {
                name: input.name_key.span(),
                requirement: input.value.span(),
            },
        ));
    }

    let inline = input.value.as_inline_table()?;
    let package_name = inline
        .get("package")
        .and_then(TomlValue::as_str)
        .unwrap_or(input.name);
    let field_value = inline_dependency_value(input.value)?;
    let registry_name = inline_registry_name(input.value);
    Some(cargo_dependency_from_span(
        CargoDependencySource {
            text: input.text,
            group: input.group,
            name: package_name,
            requirement: field_value.as_str()?,
            hosted_url: registry_name,
        },
        CargoDependencySpans {
            name: input.name_key.span(),
            requirement: field_value.span(),
        },
    ))
}
