use toml_edit::{Key, Value as TomlValue};

use versionlens_model::Dependency;

use super::spans::{PythonDependencySource, PythonDependencySpans, dependency_from_span};

pub(in crate::pyproject_toml) struct PythonKeyedDependencyInput<'a> {
    pub(in crate::pyproject_toml) text: &'a str,
    pub(in crate::pyproject_toml) group: &'a str,
    pub(in crate::pyproject_toml) name: &'a str,
    pub(in crate::pyproject_toml) value: &'a TomlValue,
    pub(in crate::pyproject_toml) key: &'a Key,
}

pub(in crate::pyproject_toml) fn keyed_dependency(
    input: PythonKeyedDependencyInput<'_>,
) -> Option<Dependency> {
    if let Some(requirement) = input.value.as_str() {
        return Some(dependency_from_span(
            PythonDependencySource {
                text: input.text,
                group: input.group,
                name: input.name,
                requirement,
                hosted_url: None,
            },
            PythonDependencySpans {
                name: input.key.span(),
                requirement: input.value.span(),
            },
        ));
    }

    let requirement = inline_dependency_requirement(input.value)?;
    Some(dependency_from_span(
        PythonDependencySource {
            text: input.text,
            group: input.group,
            name: input.name,
            requirement: requirement.as_str()?,
            hosted_url: inline_dependency_source(input.value),
        },
        PythonDependencySpans {
            name: input.key.span(),
            requirement: requirement.span(),
        },
    ))
}

fn inline_dependency_requirement(value: &TomlValue) -> Option<&TomlValue> {
    let inline = value.as_inline_table()?;
    ["version", "path", "git"]
        .into_iter()
        .find_map(|field| inline.get(field))
        .filter(|value| value.as_str().is_some())
}

fn inline_dependency_source(value: &TomlValue) -> Option<&str> {
    value
        .as_inline_table()?
        .get("source")
        .and_then(|value| value.as_str())
}
