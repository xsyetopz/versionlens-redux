use std::env::vars;
use std::fs::read_to_string;
type RegistryConfigPath = Option<PathBuf>;

fn composer_context(input: &DocumentInput) -> ComposerContext {
    ComposerContext {
        auth_entries: composer_auth_entries(input),
        repositories: parse_composer_repositories(&input.text),
        packagist_disabled: parse_composer_packagist_disabled(&input.text),
    }
}

fn composer_auth_entries(input: &DocumentInput) -> Vec<ComposerAuthEntry> {
    dot_file_texts(input, &["auth.json"])
        .iter()
        .flat_map(|text| parse_composer_auth_entries(text))
        .collect()
}

fn composer_repository_applies(repository: &ComposerRepository, name: &str) -> bool {
    (repository.only.is_empty()
        || repository
            .only
            .iter()
            .any(|pattern| composer_package_pattern_matches(pattern, name)))
        && !repository
            .exclude
            .iter()
            .any(|pattern| composer_package_pattern_matches(pattern, name))
}

fn composer_package_pattern_matches(pattern: &str, name: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "*" || pattern == name {
        return true;
    }

    pattern
        .strip_suffix('*')
        .is_some_and(|prefix| name.starts_with(prefix))
}

fn cargo_registry_source_url<'a>(
    sources: &'a [CargoRegistrySource],
    registry_name: &str,
) -> Option<&'a str> {
    let mut current = registry_name;
    for _ in 0..sources.len() {
        let source = sources.iter().find(|source| source.name == current)?;
        if let Some(replacement) = source.replace_with.as_deref() {
            current = replacement;
            continue;
        }
        return (!source.url.is_empty()).then_some(source.url.as_str());
    }

    None
}

fn cargo_config_texts(input: &DocumentInput) -> Vec<String> {
    [".cargo/config.toml", ".cargo/config"]
        .iter()
        .flat_map(|file_name| candidate_dot_file_paths(input, file_name))
        .filter_map(|path| read_to_string(path).ok())
        .collect()
}

fn npmrc_texts(
    input: &DocumentInput,
    project_npmrc_path: RegistryConfigPath,
    process_env: &[(String, String)],
) -> Vec<String> {
    let mut paths = project_npmrc_path.into_iter().collect::<Vec<_>>();
    if let Some(path) = npm_env_userconfig_path(input, process_env)
        .or_else(|| npm_default_userconfig_path(input, process_env))
    {
        push_unique_path(&mut paths, path);
    }

    paths
        .into_iter()
        .filter_map(|path| read_to_string(path).ok())
        .collect()
}

fn selected_project_yarnrc_path(input: &DocumentInput) -> RegistryConfigPath {
    selected_dot_file_path(input, ".yarnrc.yml")
        .or_else(|| selected_dot_file_path(input, ".yarnrc.yaml"))
}

fn selected_project_bunfig_path(input: &DocumentInput) -> RegistryConfigPath {
    selected_dot_file_path(input, "bunfig.toml")
        .or_else(|| selected_dot_file_path(input, ".bunfig.toml"))
}

fn dot_texts_or_candidates(
    input: &DocumentInput,
    selected_path: RegistryConfigPath,
    file_names: &[&str],
) -> Vec<String> {
    let mut paths = selected_path.into_iter().collect::<Vec<_>>();
    if paths.is_empty() {
        for file_name in file_names {
            paths.extend(candidate_dot_file_paths(input, file_name));
        }
    }
    paths
        .into_iter()
        .filter_map(|path| read_to_string(path).ok())
        .collect()
}

fn npm_env_entries(
    input: &DocumentInput,
    project_npmrc_path: Option<&PathBuf>,
    process_env: &[(String, String)],
) -> Vec<(String, String)> {
    let mut env = process_env.to_vec();
    if project_npmrc_path.is_some()
        && let Some(path) = selected_dot_file_path(input, ".env")
        && let Ok(text) = read_to_string(path)
    {
        env.extend(parse_env_entries(&text));
    }
    env
}

fn npm_env_userconfig_path(input: &DocumentInput, env: &[(String, String)]) -> RegistryConfigPath {
    let value = env_config_value(env, "NPM_CONFIG_USERCONFIG")
        .or_else(|| env_config_value(env, "npm_config_userconfig"))?
        .trim();
    if value.is_empty() {
        return None;
    }

    let path: PathBuf = value.into();
    if path.is_absolute() {
        Some(path)
    } else {
        document_parent_path(&input.uri).map(|parent| parent.join(path))
    }
}

fn npm_default_userconfig_path(input: &DocumentInput, env: &[(String, String)]) -> RegistryConfigPath {
    let parent = document_parent_path(&input.uri)?;
    if parent.parent().is_none() {
        return None;
    }

    env_config_value(env, "HOME")
        .or_else(|| env_config_value(env, "USERPROFILE"))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| {
            let path: PathBuf = value.into();
            path.join(".npmrc")
        })
}

fn dot_file_texts(input: &DocumentInput, file_names: &[&str]) -> Vec<String> {
    file_names
        .iter()
        .flat_map(|file_name| candidate_dot_file_paths(input, file_name))
        .filter_map(|path| read_to_string(path).ok())
        .collect()
}

fn env_entries(input: &DocumentInput) -> Vec<(String, String)> {
    let mut env = process_env_entries();
    env.extend(
        candidate_dot_file_paths(input, ".env")
            .into_iter()
            .filter_map(|path| read_to_string(path).ok())
            .flat_map(|text| parse_env_entries(&text)),
    );
    env
}

fn process_env_entries() -> Vec<(String, String)> {
    vars().collect()
}

fn selected_dot_file_path(input: &DocumentInput, file_name: &str) -> RegistryConfigPath {
    candidate_dot_file_paths(input, file_name)
        .into_iter()
        .find(|path| path.is_file())
}

fn candidate_dot_file_paths(input: &DocumentInput, file_name: &str) -> Vec<PathBuf> {
    let mut paths = vec![];
    if let Some(path) = document_parent_path(&input.uri) {
        push_unique_path(&mut paths, path.join(file_name));
    }
    if let Some(path) = input.workspace_root.as_deref().map(|value| {
        let path: PathBuf = value.into();
        path
    }) {
        push_unique_path(&mut paths, path.join(file_name));
    }
    paths
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn document_parent_path(uri: &str) -> RegistryConfigPath {
    document_path(uri)?.parent().map(|path| path.to_path_buf())
}

fn document_path(uri: &str) -> RegistryConfigPath {
    let path = uri.strip_prefix("file://")?;
    Some(path.into())
}

fn parse_env_entries(text: &str) -> Vec<(String, String)> {
    text.lines().filter_map(parse_env_line).collect()
}

fn parse_env_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let (key, value) = trimmed.split_once('=')?;
    let key = key.trim();
    (!key.is_empty()).then(|| (key.to_owned(), unquote_env_value(value.trim()).to_owned()))
}

fn env_config_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .rev()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

fn unquote_env_value(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
}

fn auth_registry_match_len(registry: &str, url: &str) -> Option<usize> {
    let registry = registry.trim_end_matches('/');
    if registry.is_empty() {
        return None;
    }

    ["https://", "http://"]
        .into_iter()
        .map(|scheme| format!("{scheme}{registry}"))
        .find(|prefix| url == prefix || url.starts_with(&format!("{prefix}/")))
        .map(|prefix| prefix.len())
}

fn auth_header(entry: Option<&str>) -> Vec<HttpHeader> {
    entry
        .map(|value| {
            vec![HttpHeader {
                name: "authorization".to_owned(),
                value: value.to_owned(),
                url: None,
            }]
        })
        .unwrap_or_default()
}

fn best_composer_auth_entry<'a>(entries: &'a [ComposerAuthEntry], url: &str) -> Option<&'a str> {
    entries
        .iter()
        .filter_map(|entry| auth_registry_match_len(&entry.registry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn best_maven_auth_entry<'a>(entries: &'a [MavenAuthEntry], url: &str) -> Option<&'a str> {
    entries
        .iter()
        .filter_map(|entry| full_url_or_origin_match_len(&entry.registry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn full_url_or_origin_match_len(registry: &str, url: &str) -> Option<usize> {
    let registry = registry.trim_end_matches('/');
    if registry.is_empty() {
        return None;
    }

    if url == registry || url.starts_with(&format!("{registry}/")) {
        return Some(registry.len());
    }

    registry_origin(registry)
        .filter(|origin| url == *origin || url.starts_with(&format!("{origin}/")))
        .map(str::len)
}

fn registry_origin(registry: &str) -> Option<&str> {
    let scheme_end = registry.find("://")? + 3;
    let path_start = registry[scheme_end..]
        .find('/')
        .map_or(registry.len(), |index| scheme_end + index);
    Some(&registry[..path_start])
}
