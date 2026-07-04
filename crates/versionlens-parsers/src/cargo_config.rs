use toml_edit::{Document, Item, Value as TomlValue};

#[derive(Debug, PartialEq, Eq)]
pub struct CargoRegistrySource {
    pub name: String,
    pub url: String,
    pub replace_with: Option<String>,
}

pub fn parse_cargo_config_registry_sources(text: &str) -> Vec<CargoRegistrySource> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };

    let mut sources = Vec::new();
    push_registries(&mut sources, document.get("registries"));
    push_sources(&mut sources, document.get("source"));
    sources
}

fn push_registries(out: &mut Vec<CargoRegistrySource>, item: Option<&Item>) {
    let Some(table) = item.and_then(Item::as_table) else {
        return;
    };

    out.extend(table.iter().filter_map(|(name, item)| {
        registry_url_item(item, "index").map(|url| CargoRegistrySource {
            name: name.to_owned(),
            url,
            replace_with: None,
        })
    }));
}

fn push_sources(out: &mut Vec<CargoRegistrySource>, item: Option<&Item>) {
    let Some(table) = item.and_then(Item::as_table) else {
        return;
    };

    out.extend(
        table
            .iter()
            .filter_map(|(name, item)| source_registry(name, item)),
    );
}

fn source_registry(name: &str, item: &Item) -> Option<CargoRegistrySource> {
    let url = registry_url_item(item, "registry").unwrap_or_default();
    let replace_with = string_item(item, "replace-with");
    (!url.is_empty() || replace_with.is_some()).then(|| CargoRegistrySource {
        name: name.to_owned(),
        url,
        replace_with,
    })
}

fn registry_url_item(item: &Item, field: &str) -> Option<String> {
    string_item(item, field).map(|url| {
        url.trim_start_matches("sparse+")
            .trim_end_matches('/')
            .to_owned()
    })
}

fn string_item(item: &Item, field: &str) -> Option<String> {
    item.as_table()
        .and_then(|table| table.get(field))
        .and_then(item_string_url)
        .or_else(|| {
            item.as_value()
                .and_then(TomlValue::as_inline_table)
                .and_then(|table| table.get(field))
                .and_then(value_string_url)
        })
}

fn item_string_url(item: &Item) -> Option<String> {
    item.as_value().and_then(value_string_url)
}

fn value_string_url(value: &TomlValue) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_owned)
}

#[cfg(test)]
mod tests;
