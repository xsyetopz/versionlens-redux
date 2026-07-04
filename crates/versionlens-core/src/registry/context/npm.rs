use versionlens_parsers::{
    NpmAuthEntry, NpmClientCertEntry, NpmGenericProxyConfig, NpmHttpConfig, NpmRegistryEntry,
    parse_bunfig_npm_auth_entries_with_env, parse_bunfig_npm_registry_entries_with_env,
    parse_npm_env_http_config, parse_npm_env_registry_entries, parse_npmrc_auth_entries_with_env,
    parse_npmrc_client_cert_entries_with_env, parse_npmrc_http_config_with_env,
    parse_npmrc_registry_entries_with_env,
};

#[derive(Debug, Default)]
pub(super) struct NpmContext {
    pub(super) registries: Vec<NpmRegistryEntry>,
    pub(super) auth_entries: Vec<NpmAuthEntry>,
    pub(super) client_cert_entries: Vec<NpmClientCertEntry>,
    pub(super) http: NpmHttpConfig,
}

pub(super) fn npm_context(
    npmrc_texts: &[String],
    bunfig_texts: &[String],
    env: &[(String, String)],
) -> NpmContext {
    NpmContext {
        registries: parse_npm_env_registry_entries(env)
            .into_iter()
            .chain(
                npmrc_texts
                    .iter()
                    .flat_map(|text| parse_npmrc_registry_entries_with_env(text, env)),
            )
            .chain(
                bunfig_texts
                    .iter()
                    .flat_map(|text| parse_bunfig_npm_registry_entries_with_env(text, env)),
            )
            .collect(),
        auth_entries: npmrc_texts
            .iter()
            .flat_map(|text| parse_npmrc_auth_entries_with_env(text, env))
            .chain(
                bunfig_texts
                    .iter()
                    .flat_map(|text| parse_bunfig_npm_auth_entries_with_env(text, env)),
            )
            .collect(),
        client_cert_entries: npmrc_texts
            .iter()
            .flat_map(|text| parse_npmrc_client_cert_entries_with_env(text, env))
            .collect(),
        http: npm_http_config(npmrc_texts, env),
    }
}

pub(super) fn npm_registry_entry_applies(entry: &NpmRegistryEntry, name: &str) -> bool {
    entry
        .scope
        .as_deref()
        .is_none_or(|scope| name.starts_with(&format!("{scope}/")))
}

pub(super) fn best_npm_auth_entry<'a>(entries: &'a [NpmAuthEntry], url: &str) -> Option<&'a str> {
    entries
        .iter()
        .filter_map(|entry| npm_auth_match_len(entry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn npm_http_config(npmrc_texts: &[String], env: &[(String, String)]) -> NpmHttpConfig {
    let mut merged = parse_npm_env_http_config(env);
    for config in npmrc_texts
        .iter()
        .map(|text| parse_npmrc_http_config_with_env(text, env))
    {
        if merged.strict_ssl.is_none() {
            merged.strict_ssl = config.strict_ssl;
        }
        if !merged.proxy_disabled {
            if config.proxy_disabled {
                merged.proxy = None;
                merged.proxy_disabled = true;
            } else if merged.proxy.is_none() {
                merged.proxy = config.proxy;
            }
        }
        if merged.no_proxy.is_none() {
            merged.no_proxy = config.no_proxy;
        }
        if merged.ca_file.is_none() {
            merged.ca_file = config.ca_file;
        }
        if merged.ca.is_none() {
            merged.ca = config.ca;
        }
        if merged.cert.is_none() {
            merged.cert = config.cert;
        }
        if merged.key.is_none() {
            merged.key = config.key;
        }
        if merged.timeout_ms.is_none() {
            merged.timeout_ms = config.timeout_ms;
        }
    }
    merged
}

fn npm_auth_match_len(entry: &NpmAuthEntry, url: &str) -> Option<usize> {
    super::auth_registry_match_len(entry.registry.strip_prefix("//")?, url)
}

pub(super) fn best_npm_client_cert_entry<'a>(
    entries: &'a [NpmClientCertEntry],
    url: &str,
) -> Option<&'a NpmClientCertEntry> {
    entries
        .iter()
        .filter(|entry| entry.cert_file.is_some() && entry.key_file.is_some())
        .filter_map(|entry| npm_client_cert_match_len(entry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry)
}

fn npm_client_cert_match_len(entry: &NpmClientCertEntry, url: &str) -> Option<usize> {
    super::auth_registry_match_len(entry.registry.strip_prefix("//")?, url)
}

pub(super) fn npm_generic_proxy_for_request(
    url: &str,
    generic_proxy: &NpmGenericProxyConfig,
) -> Option<String> {
    if starts_with_ignore_ascii_case(url, "https://") {
        return generic_proxy.https.as_deref().map(str::to_owned);
    }
    generic_proxy
        .https
        .as_deref()
        .or(generic_proxy.http.as_deref())
        .or(generic_proxy.plain.as_deref())
        .map(str::to_owned)
}

pub(super) fn npm_no_proxy_matches(url: &str, no_proxy: Option<&str>) -> bool {
    let Some(host) = url_host(url) else {
        return false;
    };
    no_proxy
        .into_iter()
        .flat_map(|entries| entries.split(','))
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .any(|entry| npm_no_proxy_entry_matches(host, entry))
}

fn npm_no_proxy_entry_matches(host: &str, entry: &str) -> bool {
    let host_segments = host.split('.').rev();
    let entry_segments = entry.split('.').filter(|segment| !segment.is_empty()).rev();
    let mut matched_any = false;
    for (host_segment, entry_segment) in host_segments.zip(entry_segments) {
        if !host_segment.eq_ignore_ascii_case(entry_segment) {
            return false;
        }
        matched_any = true;
    }
    matched_any
}

fn url_host(url: &str) -> Option<&str> {
    let scheme_end = url.find("://")? + 3;
    let rest = &url[scheme_end..];
    let host_end = rest.find(['/', ':', '?', '#']).unwrap_or(rest.len());
    (!rest[..host_end].is_empty()).then_some(&rest[..host_end])
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}
