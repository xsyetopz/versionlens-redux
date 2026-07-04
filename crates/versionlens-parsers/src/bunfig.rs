use toml_edit::{Document, InlineTable, Item, Table, Value as TomlValue};

use crate::npmrc::{NpmAuthEntry, NpmRegistryEntry, bearer_auth_entry, expand_env};

pub fn parse_bunfig_npm_registry_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmRegistryEntry> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(install) = document.get("install") else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    if let Some(url) = string_item(install, "registry") {
        push_registry_entry(&mut entries, None, &url, env);
    }
    push_scope_registry_entries(&mut entries, install, env);
    entries
}

pub fn parse_bunfig_npm_auth_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmAuthEntry> {
    let Ok(document) = Document::parse(text) else {
        return Vec::new();
    };
    let Some(install) = document.get("install") else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    push_scope_auth_entries(&mut entries, install, env);
    entries
}

fn push_scope_registry_entries(
    out: &mut Vec<NpmRegistryEntry>,
    install: &Item,
    env: &[(String, String)],
) {
    let Some(scopes) = table_child(install, "scopes") else {
        return;
    };

    out.extend(scopes.iter().filter_map(|(scope, item)| {
        scope_registry_url(item).map(|url| registry_entry(Some(scope), &url, env))
    }));
}

fn push_scope_auth_entries(out: &mut Vec<NpmAuthEntry>, install: &Item, env: &[(String, String)]) {
    let Some(scopes) = table_child(install, "scopes") else {
        return;
    };

    out.extend(
        scopes
            .iter()
            .filter_map(|(_, item)| scope_auth_entry(item, env)),
    );
}

fn scope_auth_entry(item: &Item, env: &[(String, String)]) -> Option<NpmAuthEntry> {
    let table = inline_table_item(item)?;
    let url = table.get("url")?.as_str()?;
    let token = table.get("token")?.as_str()?;
    bearer_auth_entry(url, token, env)
}

fn scope_registry_url(item: &Item) -> Option<String> {
    item.as_str().map(str::to_owned).or_else(|| {
        inline_table_item(item)?
            .get("url")?
            .as_str()
            .map(str::to_owned)
    })
}

fn registry_entry(scope: Option<&str>, url: &str, env: &[(String, String)]) -> NpmRegistryEntry {
    let scope = scope.map(normalize_bun_scope);
    let url = expand_env(url, env).trim().to_owned();
    NpmRegistryEntry { scope, url }
}

fn push_registry_entry(
    out: &mut Vec<NpmRegistryEntry>,
    scope: Option<&str>,
    url: &str,
    env: &[(String, String)],
) {
    let entry = registry_entry(scope, url, env);
    if !entry.url.is_empty() {
        out.push(entry);
    }
}

fn table_child<'a>(item: &'a Item, field: &str) -> Option<&'a Table> {
    item.as_table()?.get(field)?.as_table()
}

fn inline_table_item(item: &Item) -> Option<&InlineTable> {
    item.as_value().and_then(TomlValue::as_inline_table)
}

fn string_item(item: &Item, field: &str) -> Option<String> {
    item.as_table()?
        .get(field)?
        .as_str()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_owned)
}

fn normalize_bun_scope(scope: &str) -> String {
    if scope.starts_with('@') {
        scope.to_owned()
    } else {
        format!("@{scope}")
    }
}

#[cfg(test)]
mod tests;
