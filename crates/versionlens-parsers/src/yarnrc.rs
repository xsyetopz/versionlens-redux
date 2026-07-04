use marked_yaml::{parse_yaml, types::Node};

use crate::npmrc::{
    NpmAuthEntry, NpmRegistryEntry, basic_auth_entry, bearer_auth_entry, expand_env,
};

pub fn parse_yarnrc_npm_registry_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmRegistryEntry> {
    let Ok(root) = parse_yaml(0, text) else {
        return Vec::new();
    };
    let Some(root) = root.as_mapping() else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    if let Some(url) = scalar_child(root, "npmRegistryServer") {
        push_registry_entry(&mut entries, None, url, env);
    }
    collect_scope_registry_entries(root, env, &mut entries);
    entries
}

pub fn parse_yarnrc_npm_auth_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmAuthEntry> {
    let Ok(root) = parse_yaml(0, text) else {
        return Vec::new();
    };
    let Some(root) = root.as_mapping() else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    push_mapping_auth_entry(root, env, &mut entries);
    collect_scope_auth_entries(root, env, &mut entries);
    collect_registry_auth_entries(root, env, &mut entries);
    entries
}

fn collect_scope_registry_entries(
    root: &marked_yaml::types::MarkedMappingNode,
    env: &[(String, String)],
    out: &mut Vec<NpmRegistryEntry>,
) {
    let Some(Node::Mapping(scopes)) = root.get_node("npmScopes") else {
        return;
    };

    for (scope, config) in scopes.iter() {
        let Node::Mapping(config) = config else {
            continue;
        };
        let Some(url) = scalar_child(config, "npmRegistryServer") else {
            continue;
        };
        push_registry_entry(out, Some(scope.as_str()), url, env);
    }
}

fn collect_scope_auth_entries(
    root: &marked_yaml::types::MarkedMappingNode,
    env: &[(String, String)],
    out: &mut Vec<NpmAuthEntry>,
) {
    let Some(Node::Mapping(scopes)) = root.get_node("npmScopes") else {
        return;
    };

    for (_, config) in scopes.iter() {
        let Node::Mapping(config) = config else {
            continue;
        };
        push_mapping_auth_entry(config, env, out);
    }
}

fn collect_registry_auth_entries(
    root: &marked_yaml::types::MarkedMappingNode,
    env: &[(String, String)],
    out: &mut Vec<NpmAuthEntry>,
) {
    let Some(Node::Mapping(registries)) = root.get_node("npmRegistries") else {
        return;
    };

    for (url, config) in registries.iter() {
        let Node::Mapping(config) = config else {
            continue;
        };
        push_registry_mapping_auth_entry(url.as_str(), config, env, out);
    }
}

fn push_mapping_auth_entry(
    config: &marked_yaml::types::MarkedMappingNode,
    env: &[(String, String)],
    out: &mut Vec<NpmAuthEntry>,
) {
    let Some(url) = scalar_child(config, "npmRegistryServer") else {
        return;
    };
    push_registry_mapping_auth_entry(url, config, env, out);
}

fn push_registry_mapping_auth_entry(
    url: &str,
    config: &marked_yaml::types::MarkedMappingNode,
    env: &[(String, String)],
    out: &mut Vec<NpmAuthEntry>,
) {
    if let Some(token) = scalar_child(config, "npmAuthToken") {
        push_auth_entry(out, bearer_auth_entry(url, token, env));
    }
    if let Some(ident) = scalar_child(config, "npmAuthIdent") {
        push_auth_entry(out, basic_auth_entry(url, ident, env));
    }
}

fn push_auth_entry(out: &mut Vec<NpmAuthEntry>, entry: Option<NpmAuthEntry>) {
    if let Some(entry) = entry {
        out.push(entry);
    }
}

fn scalar_child<'a>(
    mapping: &'a marked_yaml::types::MarkedMappingNode,
    key: &str,
) -> Option<&'a str> {
    let Some(Node::Scalar(value)) = mapping.get_node(key) else {
        return None;
    };
    Some(value.as_str())
}

fn push_registry_entry(
    out: &mut Vec<NpmRegistryEntry>,
    scope: Option<&str>,
    url: &str,
    env: &[(String, String)],
) {
    let url = expand_env(url, env);
    let url = url.trim();
    if url.is_empty() {
        return;
    }

    out.push(NpmRegistryEntry {
        scope: scope.map(normalize_yarn_scope),
        url: url.to_owned(),
    });
}

fn normalize_yarn_scope(scope: &str) -> String {
    if scope.starts_with('@') {
        scope.to_owned()
    } else {
        format!("@{scope}")
    }
}

#[cfg(test)]
mod tests;
