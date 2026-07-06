use toml_edit::Value as TomlValue;

pub(super) struct InlineDependencyValue<'a> {
    pub(super) field: &'static str,
    pub(super) value: &'a TomlValue,
}

pub(super) fn inline_dependency_value(value: &TomlValue) -> Option<InlineDependencyValue<'_>> {
    let inline = value.as_inline_table()?;
    ["path", "git", "version"].into_iter().find_map(|field| {
        inline
            .get(field)
            .filter(|value| value.as_str().is_some())
            .map(|value| InlineDependencyValue { field, value })
    })
}

pub(super) fn inline_registry_name(value: &TomlValue) -> Option<&str> {
    value
        .as_inline_table()?
        .get("registry")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|name| !name.is_empty())
}
