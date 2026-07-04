#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmRegistryEntry {
    pub scope: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmAuthEntry {
    pub registry: String,
    pub header_value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NpmClientCertEntry {
    pub registry: String,
    pub cert_file: Option<String>,
    pub key_file: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NpmGenericProxyConfig {
    pub https: Option<String>,
    pub http: Option<String>,
    pub plain: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NpmHttpConfig {
    pub strict_ssl: Option<bool>,
    pub proxy: Option<String>,
    pub proxy_disabled: bool,
    pub no_proxy: Option<String>,
    pub generic_proxy: NpmGenericProxyConfig,
    pub ca_file: Option<String>,
    pub ca: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct NpmUsernamePasswordAuth {
    registry: String,
    username: Option<String>,
    password: Option<String>,
}

impl NpmUsernamePasswordAuth {
    fn into_entry(self) -> Option<NpmAuthEntry> {
        let username = self.username?;
        let password = self.password?;
        if username.is_empty() || password.is_empty() {
            return None;
        }
        Some(NpmAuthEntry {
            registry: self.registry,
            header_value: format!("Basic {}", base64_encode(&format!("{username}:{password}"))),
        })
    }
}

pub fn parse_npmrc_registry_entries(text: &str) -> Vec<NpmRegistryEntry> {
    parse_npmrc_registry_entries_with_env(text, &[])
}

pub fn parse_npmrc_registry_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmRegistryEntry> {
    text.lines()
        .filter_map(|line| parse_npmrc_registry_line_with_env(line, env))
        .collect()
}

pub fn parse_npmrc_auth_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmAuthEntry> {
    let mut entries = Vec::new();
    let mut username_passwords = Vec::<NpmUsernamePasswordAuth>::new();

    for line in text.lines() {
        if let Some(entry) = parse_npmrc_auth_line_with_env(line, env) {
            entries.push(entry);
            continue;
        }
        push_npmrc_username_password_auth(&mut username_passwords, line, env);
    }

    entries.extend(
        username_passwords
            .into_iter()
            .filter_map(NpmUsernamePasswordAuth::into_entry),
    );
    entries
}

pub fn parse_npmrc_client_cert_entries_with_env(
    text: &str,
    env: &[(String, String)],
) -> Vec<NpmClientCertEntry> {
    let mut entries = Vec::new();
    for line in text.lines() {
        push_npmrc_client_cert_entry(&mut entries, line, env);
    }
    entries
}

pub fn parse_npmrc_http_config_with_env(text: &str, env: &[(String, String)]) -> NpmHttpConfig {
    let mut config = NpmHttpConfig::default();
    for line in text.lines() {
        push_npmrc_http_line(&mut config, line, env);
    }
    config
}

pub fn parse_npm_env_registry_entries(env: &[(String, String)]) -> Vec<NpmRegistryEntry> {
    env.iter()
        .filter_map(|(key, value)| parse_npm_env_registry_entry(key, value))
        .collect()
}

pub fn parse_npm_env_http_config(env: &[(String, String)]) -> NpmHttpConfig {
    let mut config = NpmHttpConfig::default();
    for (key, value) in env {
        push_npm_env_http_entry(&mut config, key, value);
    }
    push_generic_proxy_env(&mut config, env);
    config
}

fn parse_npmrc_registry_line_with_env(
    line: &str,
    env: &[(String, String)],
) -> Option<NpmRegistryEntry> {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let value = expand_env(&unquote_value(value.trim()), env);
    let url = value.trim();
    if url.is_empty() {
        return None;
    }

    let key = key.trim();
    if key == "registry" {
        return Some(NpmRegistryEntry {
            scope: None,
            url: url.to_owned(),
        });
    }

    key.strip_suffix(":registry")
        .filter(|scope| scope.starts_with('@') && scope.len() > 1)
        .map(|scope| NpmRegistryEntry {
            scope: Some(scope.to_owned()),
            url: url.to_owned(),
        })
}

fn parse_npm_env_registry_entry(key: &str, value: &str) -> Option<NpmRegistryEntry> {
    let key = normalized_npm_env_key(key)?;
    let url = value.trim();
    if url.is_empty() {
        return None;
    }

    if key == "registry" {
        return Some(NpmRegistryEntry {
            scope: None,
            url: url.to_owned(),
        });
    }

    key.strip_suffix(":registry")
        .filter(|scope| scope.starts_with('@') && scope.len() > 1)
        .map(|scope| NpmRegistryEntry {
            scope: Some(scope.to_owned()),
            url: url.to_owned(),
        })
}

fn push_npmrc_http_line(config: &mut NpmHttpConfig, line: &str, env: &[(String, String)]) {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return;
    }

    let Some((key, value)) = trimmed.split_once('=') else {
        return;
    };
    let key = key.trim();
    let value = expanded_value(value, env);
    match key {
        "strict-ssl" => config.strict_ssl = npmrc_bool(&value),
        "https-proxy" if !value.is_empty() && !config.proxy_disabled => config.proxy = Some(value),
        "proxy" if value == "false" => {
            config.proxy = None;
            config.proxy_disabled = true;
        }
        "proxy" if !value.is_empty() && config.proxy.is_none() && !config.proxy_disabled => {
            config.proxy = Some(value);
        }
        "noproxy" if !value.is_empty() => config.no_proxy = Some(value),
        "cafile" if !value.is_empty() => config.ca_file = Some(value),
        "ca" | "ca[]" if !value.is_empty() => push_ca_value(config, value),
        "cert" if !value.is_empty() => config.cert = Some(value),
        "key" if !value.is_empty() => config.key = Some(value),
        "fetch-timeout" => config.timeout_ms = npmrc_timeout_ms(&value),
        _ => {}
    }
}

fn push_npm_env_http_entry(config: &mut NpmHttpConfig, key: &str, value: &str) {
    let Some(key) = normalized_npm_env_key(key) else {
        return;
    };
    let value = value.trim();
    match key.as_str() {
        "strict-ssl" => config.strict_ssl = npmrc_bool(value),
        "https-proxy" if !value.is_empty() && !config.proxy_disabled => {
            config.proxy = Some(value.to_owned());
        }
        "proxy" if value == "false" => {
            config.proxy = None;
            config.proxy_disabled = true;
        }
        "proxy" if !value.is_empty() && config.proxy.is_none() && !config.proxy_disabled => {
            config.proxy = Some(value.to_owned());
        }
        "noproxy" if !value.is_empty() => config.no_proxy = Some(value.to_owned()),
        "cafile" if !value.is_empty() => config.ca_file = Some(value.to_owned()),
        "ca" if !value.is_empty() => push_ca_value(config, value.to_owned()),
        "cert" if !value.is_empty() => config.cert = Some(value.to_owned()),
        "key" if !value.is_empty() => config.key = Some(value.to_owned()),
        "fetch-timeout" => config.timeout_ms = npmrc_timeout_ms(value),
        _ => {}
    }
}

fn push_generic_proxy_env(config: &mut NpmHttpConfig, env: &[(String, String)]) {
    if config.no_proxy.is_none() {
        config.no_proxy = env_value_ignore_ascii_case(env, "NOPROXY")
            .or_else(|| env_value_ignore_ascii_case(env, "NO_PROXY"));
    }
    if config.proxy.is_none() && !config.proxy_disabled {
        config.generic_proxy.https = env_value_ignore_ascii_case(env, "HTTPS_PROXY");
        config.generic_proxy.http = env_value_ignore_ascii_case(env, "HTTP_PROXY");
        config.generic_proxy.plain = env_value_ignore_ascii_case(env, "PROXY");
    }
}

fn push_ca_value(config: &mut NpmHttpConfig, value: String) {
    match &mut config.ca {
        Some(ca) => {
            ca.push('\n');
            ca.push_str(&value);
        }
        None => config.ca = Some(value),
    }
}

fn env_value_ignore_ascii_case(env: &[(String, String)], key: &str) -> Option<String> {
    env.iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(key))
        .map(|(_, value)| value.trim())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn normalized_npm_env_key(key: &str) -> Option<String> {
    key.strip_prefix("npm_config_")
        .or_else(|| key.strip_prefix("NPM_CONFIG_"))
        .map(str::to_ascii_lowercase)
        .map(|key| key.replace('_', "-"))
}

fn npmrc_bool(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn npmrc_timeout_ms(value: &str) -> Option<u64> {
    value.parse::<u64>().ok()
}

fn push_npmrc_client_cert_entry(
    entries: &mut Vec<NpmClientCertEntry>,
    line: &str,
    env: &[(String, String)],
) {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return;
    }

    let Some((key, value)) = trimmed.split_once('=') else {
        return;
    };
    let key = key.trim();
    if let Some(registry) = npmrc_auth_registry(key, ":certfile") {
        let cert_file = expanded_value(value, env);
        if !cert_file.is_empty() {
            npm_client_cert_entry(entries, registry).cert_file = Some(cert_file);
        }
        return;
    }
    if let Some(registry) = npmrc_auth_registry(key, ":keyfile") {
        let key_file = expanded_value(value, env);
        if !key_file.is_empty() {
            npm_client_cert_entry(entries, registry).key_file = Some(key_file);
        }
    }
}

fn npm_client_cert_entry(
    entries: &mut Vec<NpmClientCertEntry>,
    registry: String,
) -> &mut NpmClientCertEntry {
    if let Some(index) = entries.iter().position(|entry| entry.registry == registry) {
        return &mut entries[index];
    }
    entries.push(NpmClientCertEntry {
        registry,
        cert_file: None,
        key_file: None,
    });
    let index = entries.len().saturating_sub(1);
    &mut entries[index]
}

fn parse_npmrc_auth_line_with_env(line: &str, env: &[(String, String)]) -> Option<NpmAuthEntry> {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    parse_bearer_auth(key, value, env).or_else(|| parse_basic_auth(key, value, env))
}

fn push_npmrc_username_password_auth(
    auths: &mut Vec<NpmUsernamePasswordAuth>,
    line: &str,
    env: &[(String, String)],
) {
    let trimmed = line.trim();
    if ignored_line(trimmed) {
        return;
    }

    let Some((key, value)) = trimmed.split_once('=') else {
        return;
    };
    let key = key.trim();
    if let Some(registry) = npmrc_auth_registry(key, ":username") {
        let username = expanded_value(value, env);
        let auth = npm_username_password_auth(auths, registry);
        auth.username = (!username.is_empty()).then_some(username);
        return;
    }
    if let Some(registry) = npmrc_auth_registry(key, ":_password") {
        let password = decoded_npm_password(&expanded_value(value, env));
        let auth = npm_username_password_auth(auths, registry);
        auth.password = password.filter(|password| !password.is_empty());
    }
}

fn npm_username_password_auth(
    auths: &mut Vec<NpmUsernamePasswordAuth>,
    registry: String,
) -> &mut NpmUsernamePasswordAuth {
    if let Some(index) = auths.iter().position(|auth| auth.registry == registry) {
        return &mut auths[index];
    }
    auths.push(NpmUsernamePasswordAuth {
        registry,
        username: None,
        password: None,
    });
    let index = auths.len().saturating_sub(1);
    &mut auths[index]
}

fn decoded_npm_password(value: &str) -> Option<String> {
    use base64::Engine;

    if value.is_empty() {
        return None;
    }
    base64::engine::general_purpose::STANDARD
        .decode(value)
        .ok()
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
}

fn parse_bearer_auth(key: &str, value: &str, env: &[(String, String)]) -> Option<NpmAuthEntry> {
    let registry = npmrc_auth_registry(key, ":_authToken")?;
    let token = expanded_value(value, env);
    (!token.is_empty()).then(|| NpmAuthEntry {
        registry,
        header_value: format!("Bearer {token}"),
    })
}

fn parse_basic_auth(key: &str, value: &str, env: &[(String, String)]) -> Option<NpmAuthEntry> {
    let registry = npmrc_auth_registry(key, ":_auth")?;
    let token = expanded_value(value, env);
    (!token.is_empty()).then(|| NpmAuthEntry {
        registry,
        header_value: format!("Basic {token}"),
    })
}

fn npmrc_auth_registry(key: &str, suffix: &str) -> Option<String> {
    let registry = key.strip_suffix(suffix)?.trim();
    if !registry.starts_with("//") || registry.len() <= 2 {
        return None;
    }
    Some(registry.trim_end_matches('/').to_owned())
}

pub(crate) fn bearer_auth_entry(
    url: &str,
    token: &str,
    env: &[(String, String)],
) -> Option<NpmAuthEntry> {
    let registry = auth_registry_from_url(url, env)?;
    let token = expanded_value(token, env);
    (!token.is_empty()).then(|| NpmAuthEntry {
        registry,
        header_value: format!("Bearer {token}"),
    })
}

pub(crate) fn basic_auth_entry(
    url: &str,
    ident: &str,
    env: &[(String, String)],
) -> Option<NpmAuthEntry> {
    let registry = auth_registry_from_url(url, env)?;
    let ident = expanded_value(ident, env);
    if ident.is_empty() {
        return None;
    }
    Some(NpmAuthEntry {
        registry,
        header_value: format!("Basic {}", base64_encode(&ident)),
    })
}

pub(crate) fn auth_registry_from_url(url: &str, env: &[(String, String)]) -> Option<String> {
    let url = expand_env(url, env);
    let url = url.trim().trim_end_matches('/');
    let registry = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))?;
    (!registry.is_empty()).then(|| format!("//{registry}"))
}

pub(crate) fn expand_env(value: &str, env: &[(String, String)]) -> String {
    let mut expanded = String::new();
    let mut rest = value;

    while let Some(start) = rest.find("${") {
        expanded.push_str(&rest[..start]);
        let after_start = &rest[start + 2..];
        let Some(end) = after_start.find('}') else {
            expanded.push_str(&rest[start..]);
            return expanded;
        };
        let name = &after_start[..end];
        expanded.push_str(env_value(env, name).unwrap_or_default());
        rest = &after_start[end + 1..];
    }

    expanded.push_str(rest);
    expanded
}

fn expanded_value(value: &str, env: &[(String, String)]) -> String {
    expand_env(&unquote_value(value.trim()), env)
        .trim()
        .to_owned()
}

fn env_value<'a>(env: &'a [(String, String)], name: &str) -> Option<&'a str> {
    env.iter()
        .rev()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn ignored_line(trimmed: &str) -> bool {
    trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';')
}

fn base64_encode(value: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(value)
}

fn unquote_value(value: &str) -> String {
    if value.starts_with('"') && value.ends_with('"') {
        return serde_json::from_str::<String>(value)
            .unwrap_or_else(|_| value.trim_matches('"').to_owned());
    }
    value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .unwrap_or(value)
        .to_owned()
}

#[cfg(test)]
mod tests;
