use toml_edit::Value as TomlValue;

pub(super) fn inline_dependency_value(value: &TomlValue) -> Option<&TomlValue> {
    let inline = value.as_inline_table()?;
    ["version", "path", "git"]
        .into_iter()
        .find_map(|field| inline.get(field))
        .filter(|value| value.as_str().is_some())
}

pub(super) fn inline_registry_name(value: &TomlValue) -> Option<&str> {
    value
        .as_inline_table()?
        .get("registry")
        .and_then(TomlValue::as_str)
        .map(str::trim)
        .filter(|name| !name.is_empty())
}
