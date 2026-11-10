use marked_yaml::types::MarkedMappingNode;
use marked_yaml::types::Node::{Mapping as YamlMapping, Scalar as YamlScalar};

use crate::npmrc::{
    NpmAuthEntry, NpmRegistryEntry, basic_auth_entry, bearer_auth_entry, expand_env,
};
use crate::support;

type NpmAuthEntries = Vec<NpmAuthEntry>;
type YarnRegistryEntries = Vec<NpmRegistryEntry>;

pub fn parse_yarnrc_npm_registry_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmRegistryEntry> {
    support::with_yaml_mapping(text, |root| {
        let mut entries = vec![];
        if let Some(url) = scalar_child(root, "npmRegistryServer") {
            push_registry_entry(&mut entries, None, url, env);
        }
        collect_scope_registry_entries(root, env, &mut entries);
        entries
    })
    .unwrap_or_default()
}

pub fn parse_yarnrc_npm_auth_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmAuthEntry> {
    support::with_yaml_mapping(text, |root| {
        let mut entries = vec![];
        push_mapping_auth_entry(root, env, &mut entries);
        collect_scope_auth_entries(root, env, &mut entries);
        collect_registry_auth_entries(root, env, &mut entries);
        entries
    })
    .unwrap_or_default()
}

fn collect_scope_registry_entries(
    root: &MarkedMappingNode,
    env: &[(String, String)],
    out: &mut YarnRegistryEntries,
) {
    let Some(YamlMapping(scopes)) = root.get_node("npmScopes") else {
        return;
    };

    for (scope, config) in scopes.iter() {
        let YamlMapping(config) = config else {
            continue;
        };
        let Some(url) = scalar_child(config, "npmRegistryServer") else {
            continue;
        };
        push_registry_entry(out, Some(scope.as_str()), url, env);
    }
}

fn collect_scope_auth_entries(
    root: &MarkedMappingNode,
    env: &[(String, String)],
    out: &mut NpmAuthEntries,
) {
    let Some(YamlMapping(scopes)) = root.get_node("npmScopes") else {
        return;
    };

    for (_, config) in scopes.iter() {
        let YamlMapping(config) = config else {
            continue;
        };
        push_mapping_auth_entry(config, env, out);
    }
}

fn collect_registry_auth_entries(
    root: &MarkedMappingNode,
    env: &[(String, String)],
    out: &mut NpmAuthEntries,
) {
    let Some(YamlMapping(registries)) = root.get_node("npmRegistries") else {
        return;
    };

    for (url, config) in registries.iter() {
        let YamlMapping(config) = config else {
            continue;
        };
        push_registry_mapping_auth_entry(url.as_str(), config, env, out);
    }
}

fn push_mapping_auth_entry(
    config: &MarkedMappingNode,
    env: &[(String, String)],
    out: &mut NpmAuthEntries,
) {
    let Some(url) = scalar_child(config, "npmRegistryServer") else {
        return;
    };
    push_registry_mapping_auth_entry(url, config, env, out);
}

fn push_registry_mapping_auth_entry(
    url: &str,
    config: &MarkedMappingNode,
    env: &[(String, String)],
    out: &mut NpmAuthEntries,
) {
    if let Some(token) = scalar_child(config, "npmAuthToken") {
        push_auth_entry(out, bearer_auth_entry(url, token, env));
    }
    if let Some(ident) = scalar_child(config, "npmAuthIdent") {
        push_auth_entry(out, basic_auth_entry(url, ident, env));
    }
}

fn push_auth_entry(out: &mut NpmAuthEntries, entry: Option<NpmAuthEntry>) {
    if let Some(entry) = entry {
        out.push(entry);
    }
}

fn scalar_child<'a>(mapping: &'a MarkedMappingNode, key: &str) -> Option<&'a str> {
    let Some(YamlScalar(value)) = mapping.get_node(key) else {
        return None;
    };
    Some(value.as_str())
}

fn push_registry_entry(
    out: &mut YarnRegistryEntries,
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
mod tests {
use super::{parse_yarnrc_npm_auth_entries_with_env, parse_yarnrc_npm_registry_entries_with_env};
use crate::test_support::assert_registry_entries;

#[test]
fn parses_yarnrc_npm_registries_and_scope_registries() {
    let env = vec![(
        "REGISTRY_HOST".to_owned(),
        "registry.example.test".to_owned(),
    )];
    let entries = parse_yarnrc_npm_registry_entries_with_env(
        r#"
npmRegistryServer: "https://${REGISTRY_HOST}"
npmScopes:
  scope:
    npmRegistryServer: "https://scope.example.test/npm"
  '@quoted':
    npmRegistryServer: "https://quoted.example.test"
"#,
        &env,
    );

    assert_registry_entries(
        &entries,
        &[
            (None, "https://registry.example.test"),
            (Some("@scope"), "https://scope.example.test/npm"),
            (Some("@quoted"), "https://quoted.example.test"),
        ],
    );
}

#[test]
fn parses_yarnrc_npm_auth_tokens() {
    let env = vec![("TOKEN".to_owned(), "secret-token".to_owned())];
    let entries = parse_yarnrc_npm_auth_entries_with_env(
        r#"
npmRegistryServer: "https://registry.example.test"
npmAuthToken: "${TOKEN}"
npmScopes:
  scope:
    npmRegistryServer: "https://scope.example.test/npm/"
    npmAuthToken: scoped-token
npmRegistries:
  "https://other.example.test":
    npmAuthToken: other-token
"#,
        &env,
    );

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].registry, "//registry.example.test");
    assert_eq!(entries[0].header_value, "Bearer secret-token");
    assert_eq!(entries[1].registry, "//scope.example.test/npm");
    assert_eq!(entries[1].header_value, "Bearer scoped-token");
    assert_eq!(entries[2].registry, "//other.example.test");
    assert_eq!(entries[2].header_value, "Bearer other-token");
}

#[test]
fn parses_yarnrc_npm_auth_ident_as_basic_auth() {
    let env = vec![("IDENT".to_owned(), "user:pass".to_owned())];
    let entries = parse_yarnrc_npm_auth_entries_with_env(
        r#"
npmRegistryServer: "https://registry.example.test"
npmAuthIdent: "${IDENT}"
npmRegistries:
  "https://other.example.test":
    npmAuthIdent: other:pass
"#,
        &env,
    );

    crate::support::tests::assert_auth_entries(
        &entries,
        &[
            ("//registry.example.test", "Basic dXNlcjpwYXNz"),
            ("//other.example.test", "Basic b3RoZXI6cGFzcw=="),
        ],
    );
}
}
