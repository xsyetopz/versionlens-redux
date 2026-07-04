use std::path::{Path, PathBuf};

use versionlens_http::HttpHeader;
use versionlens_parsers::{
    Dependency, DocumentInput, DotnetAuthEntry, DotnetNamedSource, DotnetNugetConfig,
    DotnetSourceMapping, parse_nuget_config,
};

#[derive(Debug, Default)]
pub(super) struct DotnetContext {
    sources: Vec<DotnetNamedSource>,
    auth_entries: Vec<DotnetAuthEntry>,
    source_mappings: Vec<DotnetSourceMapping>,
    has_source_configuration: bool,
    source_inheritance_blocked: bool,
}

pub(super) fn dotnet_context(input: &DocumentInput) -> DotnetContext {
    let mut context = DotnetContext::default();
    for config in dotnet_config_files(input) {
        if let Some(parsed) = parse_nuget_config(&config.text) {
            merge_dotnet_config(&mut context, &config.path, parsed);
        }
    }
    context
}

impl DotnetContext {
    pub(super) fn has_urls(&self) -> bool {
        !self.sources.is_empty()
            || !self.auth_entries.is_empty()
            || !self.source_mappings.is_empty()
    }

    pub(super) fn has_registry_configuration(&self) -> bool {
        self.has_source_configuration || !self.source_mappings.is_empty()
    }

    pub(super) fn registry_urls(&self, dependency: &Dependency) -> Vec<String> {
        let mapped_sources = self.mapped_sources(&dependency.name);
        if !mapped_sources.is_empty() {
            return mapped_sources;
        }

        self.sources
            .iter()
            .map(|source| source.url.as_str().to_owned())
            .collect()
    }

    pub(super) fn auth_headers_for_url(&self, url: &str) -> Vec<HttpHeader> {
        super::auth_header(best_dotnet_auth_entry(&self.auth_entries, url))
    }

    fn mapped_sources(&self, name: &str) -> Vec<String> {
        if self.source_mappings.is_empty() {
            return Vec::new();
        }

        self.sources
            .iter()
            .filter(|source| dotnet_source_matches_package(&self.source_mappings, source, name))
            .map(|source| source.url.as_str().to_owned())
            .collect()
    }
}

struct DotnetConfigFile {
    path: PathBuf,
    text: String,
}

fn dotnet_config_files(input: &DocumentInput) -> Vec<DotnetConfigFile> {
    dotnet_config_dirs(input)
        .iter()
        .flat_map(|dir| {
            ["NuGet.config", "nuget.config"]
                .iter()
                .map(|file_name| dir.join(file_name))
        })
        .filter_map(|path| {
            std::fs::read_to_string(&path)
                .ok()
                .map(|text| DotnetConfigFile { path, text })
        })
        .collect()
}

fn dotnet_config_dirs(input: &DocumentInput) -> Vec<PathBuf> {
    let Some(mut current) = super::document_parent_path(&input.uri) else {
        return Vec::new();
    };
    let workspace_root = input.workspace_root.as_deref().map(Path::new);
    let mut dirs = Vec::new();

    loop {
        push_unique_path(&mut dirs, PathBuf::from(current.as_path()));
        if workspace_root.is_some_and(|root| current == root) {
            break;
        }
        if !current.pop() {
            break;
        }
    }

    dirs
}

fn merge_dotnet_config(context: &mut DotnetContext, config_path: &Path, config: DotnetNugetConfig) {
    let clear_sources = config.clear_sources;
    if config.clear_sources {
        context.sources.clear();
        context.has_source_configuration = true;
        context.source_inheritance_blocked = true;
    }
    if config.clear_auth_entries {
        context.auth_entries.clear();
    }
    if config.clear_source_mappings {
        context.source_mappings.clear();
    }
    context.has_source_configuration |=
        !config.sources.is_empty() || !config.removed_sources.is_empty();

    for name in config.removed_sources {
        context.sources.retain(|source| source.name != name);
    }
    for name in config.removed_source_mappings {
        context
            .source_mappings
            .retain(|mapping| mapping.source != name);
    }

    if !context.source_inheritance_blocked || clear_sources {
        for source in config.sources {
            let url = resolve_nuget_source_url(config_path, &source.url);
            if !remote_url(&url) {
                continue;
            }
            push_unique_source(
                &mut context.sources,
                DotnetNamedSource {
                    name: source.name,
                    url,
                },
            );
        }
    }
    context.auth_entries.extend(config.auth_entries);
    context.source_mappings.extend(config.source_mappings);
}

fn resolve_nuget_source_url(config_path: &Path, url: &str) -> String {
    if remote_url(url) || url.starts_with("file://") || Path::new(url).is_absolute() {
        return url.to_owned();
    }

    config_path
        .parent()
        .map(|parent| parent.join(url).to_string_lossy().into_owned())
        .unwrap_or_else(|| url.to_owned())
}

fn remote_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn best_dotnet_auth_entry<'a>(entries: &'a [DotnetAuthEntry], url: &str) -> Option<&'a str> {
    entries
        .iter()
        .filter_map(|entry| dotnet_auth_match_len(entry, url).map(|len| (entry, len)))
        .max_by_key(|(_, len)| *len)
        .map(|(entry, _)| entry.header_value.as_str())
}

fn dotnet_auth_match_len(entry: &DotnetAuthEntry, url: &str) -> Option<usize> {
    super::full_url_or_origin_match_len(&entry.registry, url)
}

fn dotnet_source_matches_package(
    mappings: &[DotnetSourceMapping],
    source: &DotnetNamedSource,
    name: &str,
) -> bool {
    mappings.iter().any(|mapping| {
        mapping.source == source.name && dotnet_package_pattern_matches(&mapping.pattern, name)
    })
}

fn dotnet_package_pattern_matches(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    pattern.strip_suffix('*').map_or_else(
        || pattern.eq_ignore_ascii_case(name),
        |prefix| name_prefix_matches(name, prefix),
    )
}

fn name_prefix_matches(name: &str, prefix: &str) -> bool {
    name.get(..prefix.len())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(prefix))
}

fn push_unique_source(sources: &mut Vec<DotnetNamedSource>, source: DotnetNamedSource) {
    if !sources
        .iter()
        .any(|existing: &DotnetNamedSource| existing.url == source.url)
    {
        sources.push(source);
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}
