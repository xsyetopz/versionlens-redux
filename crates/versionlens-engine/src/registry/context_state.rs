use std::env::vars;

use crate::workspace::workspace_path;

type RegistryConfigPath = Option<PathBuf>;

fn composer_context(input: &DocumentInput, files: &impl RegistryFileRead) -> ComposerContext {
    ComposerContext {
        auth_entries: composer_auth_entries(input, files),
        repositories: parse_composer_repositories(&input.text),
        packagist_disabled: parse_composer_packagist_disabled(&input.text),
    }
}

fn composer_auth_entries(
    input: &DocumentInput,
    files: &impl RegistryFileRead,
) -> Vec<ComposerAuthEntry> {
    dot_file_texts(input, &["auth.json"], files)
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

fn cargo_config_texts(input: &DocumentInput, files: &impl RegistryFileRead) -> Vec<String> {
    [".cargo/config.toml", ".cargo/config"]
        .iter()
        .flat_map(|file_name| candidate_dot_file_paths(input, file_name))
        .filter_map(|path| files.read(&path))
        .collect()
}

fn npmrc_texts(
    input: &DocumentInput,
    project_npmrc_path: RegistryConfigPath,
    process_env: &[(String, String)],
    files: &impl RegistryFileRead,
) -> Vec<String> {
    let mut paths = project_npmrc_path.into_iter().collect::<Vec<_>>();
    let required_userconfig = npm_env_userconfig_path(input, process_env);
    if let Some(path) = required_userconfig
        .clone()
        .or_else(|| npm_default_userconfig_path(input, process_env))
    {
        push_unique_path(&mut paths, path);
    }

    paths
        .into_iter()
        .filter_map(|path| {
            if required_userconfig.as_ref() == Some(&path) {
                files.read_required(&path)
            } else {
                files.read(&path)
            }
        })
        .collect()
}

fn selected_project_yarnrc_path(
    input: &DocumentInput,
    files: &impl RegistryFileRead,
) -> RegistryConfigPath {
    selected_dot_file_path(input, ".yarnrc.yml", files)
        .or_else(|| selected_dot_file_path(input, ".yarnrc.yaml", files))
}

fn selected_project_bunfig_path(
    input: &DocumentInput,
    files: &impl RegistryFileRead,
) -> RegistryConfigPath {
    selected_dot_file_path(input, "bunfig.toml", files)
        .or_else(|| selected_dot_file_path(input, ".bunfig.toml", files))
}

fn dot_texts_or_candidates(
    input: &DocumentInput,
    selected_path: RegistryConfigPath,
    file_names: &[&str],
    files: &impl RegistryFileRead,
) -> Vec<String> {
    let mut paths = selected_path.into_iter().collect::<Vec<_>>();
    if paths.is_empty() {
        for file_name in file_names {
            paths.extend(candidate_dot_file_paths(input, file_name));
        }
    }
    paths
        .into_iter()
        .filter_map(|path| files.read(&path))
        .collect()
}

fn npm_env_entries(
    input: &DocumentInput,
    project_npmrc_path: Option<&PathBuf>,
    process_env: &[(String, String)],
    files: &impl RegistryFileRead,
) -> Vec<(String, String)> {
    let mut env = process_env.to_vec();
    if project_npmrc_path.is_some()
        && let Some(path) = selected_dot_file_path(input, ".env", files)
        && let Some(text) = files.read(&path)
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

fn npm_default_userconfig_path(
    input: &DocumentInput,
    env: &[(String, String)],
) -> RegistryConfigPath {
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

fn dot_file_texts(
    input: &DocumentInput,
    file_names: &[&str],
    files: &impl RegistryFileRead,
) -> Vec<String> {
    file_names
        .iter()
        .flat_map(|file_name| candidate_dot_file_paths(input, file_name))
        .filter_map(|path| files.read(&path))
        .collect()
}

fn env_entries(input: &DocumentInput, files: &impl RegistryFileRead) -> Vec<(String, String)> {
    let mut env = process_env_entries();
    env.extend(
        candidate_dot_file_paths(input, ".env")
            .into_iter()
            .filter_map(|path| files.read(&path))
            .flat_map(|text| parse_env_entries(&text)),
    );
    env
}

fn process_env_entries() -> Vec<(String, String)> {
    vars().collect()
}

fn selected_dot_file_path(
    input: &DocumentInput,
    file_name: &str,
    files: &impl RegistryFileRead,
) -> RegistryConfigPath {
    candidate_dot_file_paths(input, file_name)
        .into_iter()
        .find(|path| files.read(path).is_some())
}

fn candidate_dot_file_paths(input: &DocumentInput, file_name: &str) -> Vec<PathBuf> {
    let mut paths = vec![];
    if let Some(path) = document_parent_path(&input.uri) {
        push_unique_path(&mut paths, path.join(file_name));
    }
    if let Some(path) = input.workspace_root.as_deref().and_then(workspace_path) {
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
    workspace_path(uri)?.parent().map(|path| path.to_path_buf())
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
    (!key.is_empty()).then(|| {
        (
            key.to_owned(),
            versionlens_model::strip_matching_quotes(value.trim()).to_owned(),
        )
    })
}

fn env_config_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a str> {
    env.iter()
        .rev()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
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

pub(super) fn best_matching_auth_entry<'a, T>(
    entries: &'a [T],
    url: &str,
    registry: fn(&T) -> &str,
    match_len: fn(&str, &str) -> Option<usize>,
) -> Option<&'a T> {
    entries
        .iter()
        .filter_map(|entry| match_len(registry(entry), url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry)
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
    best_matching_auth_entry(
        entries,
        url,
        |entry| &entry.registry,
        auth_registry_match_len,
    )
    .map(|entry| entry.header_value.as_str())
}

fn best_maven_auth_entry<'a>(entries: &'a [MavenAuthEntry], url: &str) -> Option<&'a str> {
    best_matching_auth_entry(
        entries,
        url,
        |entry| &entry.registry,
        full_url_or_origin_match_len,
    )
    .map(|entry| entry.header_value.as_str())
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
