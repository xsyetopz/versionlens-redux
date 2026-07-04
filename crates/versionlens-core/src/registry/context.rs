use std::path::{Path, PathBuf};

use versionlens_http::{HttpConfig, HttpHeader};
use versionlens_parsers::{
    CargoRegistrySource, ComposerAuthEntry, ComposerRepository, Dependency, DocumentInput,
    Ecosystem, ManifestKind, MavenAuthEntry, MavenMirror, MavenNamedRepository,
    parse_cargo_config_registry_sources, parse_composer_auth_entries,
    parse_composer_packagist_disabled, parse_composer_repositories, parse_gemfile_source_urls,
    parse_go_proxy_urls, parse_maven_pom_repositories, parse_maven_settings_auth_entries,
    parse_maven_settings_mirror_urls, parse_maven_settings_mirrors,
    parse_maven_settings_repositories,
};
use versionlens_providers::registry_url_with_base;

use crate::RegistryUrlConfig;

mod dotnet;
mod npm;

use dotnet::{DotnetContext, dotnet_context};
use npm::{
    NpmContext, best_npm_auth_entry, best_npm_client_cert_entry, npm_context,
    npm_generic_proxy_for_request, npm_no_proxy_matches, npm_registry_entry_applies,
};

#[derive(Debug, Default)]
pub(crate) struct RegistryContext {
    manifest_kind: Option<ManifestKind>,
    pub(crate) urls: Vec<RegistryUrlConfig>,
    composer: ComposerContext,
    npm: NpmContext,
    dotnet: DotnetContext,
    maven_auth_entries: Vec<MavenAuthEntry>,
    maven_uses_mirror: bool,
    cargo_registries: Vec<CargoRegistrySource>,
}

#[derive(Debug, Default)]
struct ComposerContext {
    auth_entries: Vec<ComposerAuthEntry>,
    repositories: Vec<ComposerRepository>,
    packagist_disabled: bool,
}

impl RegistryContext {
    #[cfg(test)]
    pub(crate) fn from_document(input: &DocumentInput) -> Self {
        let kind = versionlens_parsers::classify_document(input);
        Self::from_document_kind(input, kind)
    }

    pub(crate) fn from_document_kind(input: &DocumentInput, kind: ManifestKind) -> Self {
        let mut context = match kind {
            ManifestKind::CargoToml => Self::from_cargo_document(input),
            ManifestKind::ComposerJson => Self::from_composer_document(input),
            ManifestKind::DotnetProjectJson | ManifestKind::DotnetXml => {
                Self::from_dotnet_document(input)
            }
            ManifestKind::Gemfile => Self::from_ruby_document(input),
            ManifestKind::GoMod => Self::from_go_document(input),
            ManifestKind::MavenPomXml => Self::from_maven_document(input),
            ManifestKind::DenoJson | ManifestKind::NpmPackageJson | ManifestKind::PnpmYaml => {
                Self::from_npm_document(input)
            }
            ManifestKind::PythonPipfile
            | ManifestKind::PythonPyprojectToml
            | ManifestKind::PythonRequirementsTxt => Self::from_python_document(input),
            _ => Self::default(),
        };
        context.manifest_kind = Some(kind);
        context
    }

    pub(crate) fn manifest_kind(&self) -> Option<ManifestKind> {
        self.manifest_kind
    }

    fn from_composer_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: Vec::new(),
            composer: composer_context(input),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }

    fn from_cargo_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: Vec::new(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: cargo_config_texts(input)
                .iter()
                .flat_map(|text| parse_cargo_config_registry_sources(text))
                .collect(),
        }
    }

    fn from_dotnet_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: Vec::new(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: dotnet_context(input),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }

    fn from_ruby_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: parse_gemfile_source_urls(&input.text)
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Ecosystem::Ruby,
                    url,
                })
                .collect(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }

    fn from_go_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: parse_go_proxy_urls(&env_entries(input))
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Ecosystem::Go,
                    url,
                })
                .collect(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }

    fn from_maven_document(input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: parse_maven_registry_urls(input)
                .into_iter()
                .map(|url| RegistryUrlConfig {
                    ecosystem: Ecosystem::Maven,
                    url,
                })
                .collect(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: maven_auth_entries(input),
            maven_uses_mirror: maven_uses_mirror(input),
            cargo_registries: Vec::new(),
        }
    }

    pub(crate) fn has_urls(&self) -> bool {
        !self.urls.is_empty()
            || !self.composer.auth_entries.is_empty()
            || !self.composer.repositories.is_empty()
            || !self.npm.registries.is_empty()
            || !self.npm.auth_entries.is_empty()
            || !self.npm.client_cert_entries.is_empty()
            || self.npm.http.strict_ssl.is_some()
            || self.npm.http.proxy.is_some()
            || self.npm.http.no_proxy.is_some()
            || self.npm.http.proxy_disabled
            || self.npm.http.ca_file.is_some()
            || self.npm.http.ca.is_some()
            || self.npm.http.cert.is_some()
            || self.npm.http.key.is_some()
            || self.dotnet.has_urls()
            || !self.maven_auth_entries.is_empty()
            || self.maven_uses_mirror
            || !self.cargo_registries.is_empty()
    }

    pub(crate) fn maven_uses_mirror(&self) -> bool {
        self.maven_uses_mirror
    }

    pub(crate) fn default_registry_disabled(&self, ecosystem: Ecosystem) -> bool {
        ecosystem == Ecosystem::Composer && self.composer.packagist_disabled
    }

    pub(crate) fn has_dotnet_registry_configuration(&self) -> bool {
        self.dotnet.has_registry_configuration()
    }

    pub(crate) fn dotnet_registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.dotnet.registry_urls(dependency)
    }

    pub(crate) fn registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        if dependency.ecosystem == Ecosystem::Cargo {
            return self.cargo_registry_urls(dependency);
        }

        if dependency.ecosystem == Ecosystem::Composer {
            return self.composer_registry_urls(dependency);
        }

        if dependency.ecosystem != Ecosystem::Npm {
            return Vec::new();
        }

        if let Some(url) = self
            .npm
            .registries
            .iter()
            .find(|entry| {
                entry.scope.is_some() && npm_registry_entry_applies(entry, &dependency.name)
            })
            .map(|entry| registry_url_with_base(Ecosystem::Npm, &dependency.name, Some(&entry.url)))
        {
            return vec![url];
        }

        self.npm
            .registries
            .iter()
            .find(|entry| entry.scope.is_none())
            .map(|entry| registry_url_with_base(Ecosystem::Npm, &dependency.name, Some(&entry.url)))
            .into_iter()
            .collect()
    }

    fn composer_registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        self.composer
            .repositories
            .iter()
            .filter(|repository| composer_repository_applies(repository, &dependency.name))
            .map(|repository| {
                registry_url_with_base(Ecosystem::Composer, &dependency.name, Some(&repository.url))
            })
            .collect()
    }

    fn cargo_registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        let registry_name = dependency.hosted_url.as_deref().unwrap_or("crates-io");
        let Some(url) = cargo_registry_source_url(&self.cargo_registries, registry_name) else {
            return Vec::new();
        };

        vec![registry_url_with_base(
            Ecosystem::Cargo,
            &dependency.name,
            Some(url),
        )]
    }

    pub(crate) fn auth_headers_for_url(&self, ecosystem: Ecosystem, url: &str) -> Vec<HttpHeader> {
        match ecosystem {
            Ecosystem::Composer => {
                auth_header(best_composer_auth_entry(&self.composer.auth_entries, url))
            }
            Ecosystem::Dotnet => self.dotnet.auth_headers_for_url(url),
            Ecosystem::Maven => auth_header(best_maven_auth_entry(&self.maven_auth_entries, url)),
            Ecosystem::Npm => auth_header(best_npm_auth_entry(&self.npm.auth_entries, url)),
            _ => Vec::new(),
        }
    }

    pub(crate) fn http_config_for_request(
        &self,
        ecosystem: Ecosystem,
        url: &str,
        base: HttpConfig,
    ) -> HttpConfig {
        if ecosystem != Ecosystem::Npm {
            return base;
        }

        let is_npm_registry_fetch =
            !Self::starts_with_ignore_ascii_case(url, "https://api.github.com/repos/");
        let client_cert = best_npm_client_cert_entry(&self.npm.client_cert_entries, url);
        HttpConfig {
            timeout_ms: if is_npm_registry_fetch {
                npm_registry_timeout_ms(self.npm.http.timeout_ms)
            } else {
                base.timeout_ms
            },
            strict_ssl: self.npm.http.strict_ssl.unwrap_or(base.strict_ssl),
            proxy: self.proxy_for_request(url, base.proxy),
            ca_file: self
                .npm
                .http
                .ca_file
                .as_deref()
                .map(str::to_owned)
                .or(base.ca_file),
            ca: self.npm.http.ca.as_deref().map(str::to_owned).or(base.ca),
            cert_file: client_cert
                .and_then(|entry| entry.cert_file.as_deref())
                .map(str::to_owned)
                .or(base.cert_file),
            key_file: client_cert
                .and_then(|entry| entry.key_file.as_deref())
                .map(str::to_owned)
                .or(base.key_file),
            cert: client_cert
                .is_none()
                .then(|| self.npm.http.cert.as_deref())
                .flatten()
                .map(str::to_owned)
                .or(base.cert),
            key: client_cert
                .is_none()
                .then(|| self.npm.http.key.as_deref())
                .flatten()
                .map(str::to_owned)
                .or(base.key),
            auth_headers: base.auth_headers,
        }
    }

    fn proxy_for_request(&self, url: &str, base_proxy: Option<String>) -> Option<String> {
        if self.npm.http.proxy_disabled
            || npm_no_proxy_matches(url, self.npm.http.no_proxy.as_deref())
        {
            return None;
        }

        self.npm
            .http
            .proxy
            .as_deref()
            .map(str::to_owned)
            .or_else(|| npm_generic_proxy_for_request(url, &self.npm.http.generic_proxy))
            .or(base_proxy)
    }

    fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
        value
            .get(..prefix.len())
            .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
    }

    fn from_python_document(_input: &DocumentInput) -> Self {
        Self {
            manifest_kind: None,
            urls: Vec::new(),
            composer: ComposerContext::default(),
            npm: NpmContext::default(),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }

    fn from_npm_document(input: &DocumentInput) -> Self {
        let process_env = process_env_entries();
        let project_npmrc_path = selected_project_npmrc_path(input);
        let env = npm_env_entries(input, project_npmrc_path.as_ref(), &process_env);
        let npmrc_texts = npmrc_texts(input, project_npmrc_path, &process_env);
        let bunfig_texts = dot_file_texts(input, &["bunfig.toml", ".bunfig.toml"]);
        Self {
            manifest_kind: None,
            urls: Vec::new(),
            composer: ComposerContext::default(),
            npm: npm_context(&npmrc_texts, &bunfig_texts, &env),
            dotnet: DotnetContext::default(),
            maven_auth_entries: Vec::new(),
            maven_uses_mirror: false,
            cargo_registries: Vec::new(),
        }
    }
}

fn npm_registry_timeout_ms(timeout_ms: Option<u64>) -> u64 {
    match timeout_ms {
        Some(0) => 30_000,
        Some(timeout_ms) => timeout_ms,
        None => 300_000,
    }
}

fn maven_auth_entries(input: &DocumentInput) -> Vec<MavenAuthEntry> {
    dot_file_texts(input, &["settings.xml"])
        .iter()
        .flat_map(|text| parse_maven_settings_auth_entries(text))
        .collect()
}

fn maven_uses_mirror(input: &DocumentInput) -> bool {
    dot_file_texts(input, &["settings.xml"])
        .iter()
        .any(|text| !parse_maven_settings_mirror_urls(text).is_empty())
}

fn parse_maven_registry_urls(input: &DocumentInput) -> Vec<String> {
    let settings_texts = dot_file_texts(input, &["settings.xml"]);
    let mirrors = settings_texts
        .iter()
        .flat_map(|text| parse_maven_settings_mirrors(text))
        .collect::<Vec<_>>();
    let mirror_urls = all_repository_mirror_urls(&mirrors);
    if !mirror_urls.is_empty() {
        return mirror_urls;
    }

    let repositories = parse_maven_pom_repositories(&input.text).into_iter().chain(
        settings_texts
            .iter()
            .flat_map(|text| parse_maven_settings_repositories(text)),
    );

    mirrored_maven_repository_urls(repositories, &mirrors)
}

fn all_repository_mirror_urls(mirrors: &[MavenMirror]) -> Vec<String> {
    mirrors
        .iter()
        .filter(|mirror| mirror.mirror_of == "*")
        .map(|mirror| mirror.url.as_str().to_owned())
        .collect()
}

fn mirrored_maven_repository_urls(
    repositories: impl Iterator<Item = MavenNamedRepository>,
    mirrors: &[MavenMirror],
) -> Vec<String> {
    repositories
        .map(|repository| mirrored_maven_repository_url(repository, mirrors))
        .collect()
}

fn mirrored_maven_repository_url(
    repository: MavenNamedRepository,
    mirrors: &[MavenMirror],
) -> String {
    mirrors
        .iter()
        .find(|mirror| mirror.mirror_of == repository.id)
        .map_or(repository.url, |mirror| mirror.url.as_str().to_owned())
}

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
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect()
}

fn npmrc_texts(
    input: &DocumentInput,
    project_npmrc_path: Option<PathBuf>,
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
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect()
}

fn selected_project_npmrc_path(input: &DocumentInput) -> Option<PathBuf> {
    selected_dot_file_path(input, ".npmrc")
}

fn npm_env_entries(
    input: &DocumentInput,
    project_npmrc_path: Option<&PathBuf>,
    process_env: &[(String, String)],
) -> Vec<(String, String)> {
    let mut env = process_env.to_vec();
    if project_npmrc_path.is_some()
        && let Some(path) = selected_dot_file_path(input, ".env")
        && let Ok(text) = std::fs::read_to_string(path)
    {
        env.extend(parse_env_entries(&text));
    }
    env
}

fn npm_env_userconfig_path(input: &DocumentInput, env: &[(String, String)]) -> Option<PathBuf> {
    let value = env_config_value(env, "NPM_CONFIG_USERCONFIG")
        .or_else(|| env_config_value(env, "npm_config_userconfig"))?
        .trim();
    if value.is_empty() {
        return None;
    }

    let path = PathBuf::from(value);
    if path.is_absolute() {
        Some(path)
    } else {
        document_parent_path(&input.uri).map(|parent| parent.join(path))
    }
}

fn npm_default_userconfig_path(input: &DocumentInput, env: &[(String, String)]) -> Option<PathBuf> {
    let parent = document_parent_path(&input.uri)?;
    if parent.parent().is_none() {
        return None;
    }

    env_config_value(env, "HOME")
        .or_else(|| env_config_value(env, "USERPROFILE"))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| PathBuf::from(value).join(".npmrc"))
}

fn dot_file_texts(input: &DocumentInput, file_names: &[&str]) -> Vec<String> {
    dot_file_texts_except(input, file_names, None)
}

fn dot_file_texts_except(
    input: &DocumentInput,
    file_names: &[&str],
    excluded_path: Option<&Path>,
) -> Vec<String> {
    file_names
        .iter()
        .flat_map(|file_name| candidate_dot_file_paths(input, file_name))
        .filter(|path| excluded_path.is_none_or(|excluded| path != excluded))
        .filter_map(|path| std::fs::read_to_string(path).ok())
        .collect()
}

fn env_entries(input: &DocumentInput) -> Vec<(String, String)> {
    let mut env = process_env_entries();
    env.extend(
        candidate_dot_file_paths(input, ".env")
            .into_iter()
            .filter_map(|path| std::fs::read_to_string(path).ok())
            .flat_map(|text| parse_env_entries(&text)),
    );
    env
}

fn process_env_entries() -> Vec<(String, String)> {
    std::env::vars().collect()
}

fn selected_dot_file_path(input: &DocumentInput, file_name: &str) -> Option<PathBuf> {
    candidate_dot_file_paths(input, file_name)
        .into_iter()
        .find(|path| path.is_file())
}

fn candidate_dot_file_paths(input: &DocumentInput, file_name: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = document_parent_path(&input.uri) {
        push_unique_path(&mut paths, path.join(file_name));
    }
    if let Some(path) = input.workspace_root.as_deref().map(PathBuf::from) {
        push_unique_path(&mut paths, path.join(file_name));
    }
    paths
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn document_parent_path(uri: &str) -> Option<PathBuf> {
    document_path(uri)?.parent().map(Path::to_path_buf)
}

fn document_path(uri: &str) -> Option<PathBuf> {
    let path = uri.strip_prefix("file://")?;
    Some(PathBuf::from(path))
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
        .filter_map(|entry| composer_auth_match_len(entry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn composer_auth_match_len(entry: &ComposerAuthEntry, url: &str) -> Option<usize> {
    auth_registry_match_len(&entry.registry, url)
}

fn best_maven_auth_entry<'a>(entries: &'a [MavenAuthEntry], url: &str) -> Option<&'a str> {
    entries
        .iter()
        .filter_map(|entry| maven_auth_match_len(entry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn maven_auth_match_len(entry: &MavenAuthEntry, url: &str) -> Option<usize> {
    full_url_or_origin_match_len(&entry.registry, url)
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
