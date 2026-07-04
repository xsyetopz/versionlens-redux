use std::collections::{HashMap, HashSet};

use base64::{Engine, engine::general_purpose::STANDARD};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use super::model::{DotnetAuthEntry, DotnetNamedSource, DotnetNugetConfig, DotnetSourceMapping};
use super::protocol::protocol_from_url;

#[derive(Debug, PartialEq, Eq)]
struct NugetConfigSource {
    name: String,
    url: String,
}

#[derive(Debug, Default)]
struct NugetConfigCredentials {
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Default)]
struct NugetConfigState {
    section: NugetConfigSection,
    credential_source: Option<String>,
    mapping_source: Option<String>,
    sources: Vec<NugetConfigSource>,
    disabled: HashSet<String>,
    credentials: HashMap<String, NugetConfigCredentials>,
    source_mappings: Vec<DotnetSourceMapping>,
    changes: NugetConfigChanges,
}

#[derive(Debug, Default)]
struct NugetConfigChanges {
    removed_sources: HashSet<String>,
    removed_source_mappings: HashSet<String>,
    clear_sources: bool,
    clear_auth_entries: bool,
    clear_source_mappings: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum NugetConfigSection {
    DisabledPackageSources,
    PackageSourceCredentials,
    PackageSourceMapping,
    PackageSources,
    #[default]
    Other,
}

pub fn parse_nuget_config_source_urls(text: &str) -> Vec<String> {
    parse_nuget_config_named_sources(text)
        .into_iter()
        .map(|source| source.url)
        .collect()
}

pub fn parse_nuget_config_named_sources(text: &str) -> Vec<DotnetNamedSource> {
    parse_nuget_config(text)
        .map(|config| config.sources)
        .unwrap_or_default()
}

pub fn parse_nuget_config_source_mappings(text: &str) -> Vec<DotnetSourceMapping> {
    parse_nuget_config(text)
        .map(|config| config.source_mappings)
        .unwrap_or_default()
}

pub fn parse_nuget_config_auth_entries(text: &str) -> Vec<DotnetAuthEntry> {
    parse_nuget_config(text)
        .map(|config| config.auth_entries)
        .unwrap_or_default()
}

pub fn parse_nuget_config(text: &str) -> Option<DotnetNugetConfig> {
    parse_nuget_config_state(text).map(dotnet_nuget_config)
}

fn parse_nuget_config_state(text: &str) -> Option<NugetConfigState> {
    let mut reader = Reader::from_str(text);
    let mut state = NugetConfigState::default();

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => start_tag(&mut state, &event),
            Ok(Event::End(event)) => end_tag(&mut state, event.name().as_ref()),
            Ok(Event::Empty(event)) => collect_empty_tag(&mut state, &event),
            Ok(Event::Eof) => break,
            Err(_) => return None,
            _ => {}
        }
    }

    Some(state)
}

fn dotnet_nuget_config(state: NugetConfigState) -> DotnetNugetConfig {
    let sources = enabled_sources(&state.sources, &state.disabled);
    let auth_entries = auth_entries(&state.sources, &state.disabled, &state.credentials);
    DotnetNugetConfig {
        sources,
        auth_entries,
        source_mappings: state.source_mappings,
        removed_sources: state.changes.removed_sources.into_iter().collect(),
        removed_source_mappings: state.changes.removed_source_mappings.into_iter().collect(),
        clear_sources: state.changes.clear_sources,
        clear_auth_entries: state.changes.clear_auth_entries,
        clear_source_mappings: state.changes.clear_source_mappings,
    }
}

fn enabled_sources(
    sources: &[NugetConfigSource],
    disabled: &HashSet<String>,
) -> Vec<DotnetNamedSource> {
    sources
        .iter()
        .filter(|source| !disabled.contains(&source.name))
        .map(|source| DotnetNamedSource {
            name: source.name.as_str().to_owned(),
            url: source.url.as_str().to_owned(),
        })
        .collect()
}

fn auth_entries(
    sources: &[NugetConfigSource],
    disabled: &HashSet<String>,
    credentials: &HashMap<String, NugetConfigCredentials>,
) -> Vec<DotnetAuthEntry> {
    sources
        .iter()
        .filter(|source| !disabled.contains(&source.name))
        .filter(|source| is_remote_url(&source.url))
        .filter_map(|source| auth_entry_from_source(source, credentials))
        .collect()
}

fn start_tag(state: &mut NugetConfigState, event: &BytesStart<'_>) {
    match section_from_event(event) {
        NugetConfigSection::Other
            if state.section == NugetConfigSection::PackageSourceCredentials =>
        {
            state.credential_source = event_name(event);
        }
        NugetConfigSection::Other
            if state.section == NugetConfigSection::PackageSourceMapping
                && event_name_is(event, "packageSource") =>
        {
            state.mapping_source = attr_value(event, "key");
        }
        section => state.section = section,
    }
}

fn end_tag(state: &mut NugetConfigState, name: &[u8]) {
    if state
        .credential_source
        .as_deref()
        .is_some_and(|source| source.as_bytes() == name)
    {
        state.credential_source = None;
        return;
    }

    if name == b"packageSource" && state.section == NugetConfigSection::PackageSourceMapping {
        state.mapping_source = None;
        return;
    }

    if matches!(
        name,
        b"disabledPackageSources"
            | b"packageSourceCredentials"
            | b"packageSourceMapping"
            | b"packageSources"
    ) {
        state.section = NugetConfigSection::Other;
    }
}

fn collect_empty_tag(state: &mut NugetConfigState, event: &BytesStart<'_>) {
    if event_name_is(event, "clear") {
        clear_section(state);
        return;
    }
    if event_name_is(event, "remove") {
        remove_section_entry(state, event);
        return;
    }

    match state.section {
        NugetConfigSection::PackageSources => {
            if event_name_is(event, "add")
                && let Some(source) = source_from_add(event)
            {
                state.sources.push(source);
            }
        }
        NugetConfigSection::DisabledPackageSources => {
            if event_name_is(event, "add")
                && add_value_is_true(event)
                && let Some(name) = attr_value(event, "key")
            {
                state.disabled.insert(name);
            }
        }
        NugetConfigSection::PackageSourceCredentials => {
            if event_name_is(event, "add")
                && let Some(source) = &state.credential_source
            {
                collect_credential(&mut state.credentials, source, event);
            }
        }
        NugetConfigSection::PackageSourceMapping => {
            if event_name_is(event, "package")
                && let Some(source) = &state.mapping_source
                && let Some(mapping) = source_mapping_from_package(source, event)
            {
                state.source_mappings.push(mapping);
            }
        }
        NugetConfigSection::Other => {}
    }
}

fn remove_section_entry(state: &mut NugetConfigState, event: &BytesStart<'_>) {
    let Some(key) = attr_value(event, "key") else {
        return;
    };

    match state.section {
        NugetConfigSection::PackageSources => {
            state.sources.retain(|source| source.name != key);
            state.changes.removed_sources.insert(key);
        }
        NugetConfigSection::DisabledPackageSources => {
            state.disabled.remove(&key);
        }
        NugetConfigSection::PackageSourceCredentials => {
            if let Some(source) = &state.credential_source
                && let Some(credentials) = state.credentials.get_mut(source)
            {
                remove_credential(credentials, &key);
            }
        }
        NugetConfigSection::PackageSourceMapping => {
            state
                .source_mappings
                .retain(|mapping| mapping.source != key);
            state.changes.removed_source_mappings.insert(key);
        }
        NugetConfigSection::Other => {}
    }
}

fn clear_section(state: &mut NugetConfigState) {
    match state.section {
        NugetConfigSection::PackageSources => {
            state.sources.clear();
            state.changes.clear_sources = true;
        }
        NugetConfigSection::DisabledPackageSources => state.disabled.clear(),
        NugetConfigSection::PackageSourceCredentials => {
            state.credentials.clear();
            state.changes.clear_auth_entries = true;
        }
        NugetConfigSection::PackageSourceMapping => {
            state.source_mappings.clear();
            state.changes.clear_source_mappings = true;
        }
        NugetConfigSection::Other => {}
    }
}

fn remove_credential(credentials: &mut NugetConfigCredentials, key: &str) {
    match key {
        "Username" => credentials.username = None,
        "ClearTextPassword" => credentials.password = None,
        _ => {}
    }
}

fn collect_credential(
    credentials: &mut HashMap<String, NugetConfigCredentials>,
    source: &str,
    event: &BytesStart<'_>,
) {
    let Some(key) = attr_value(event, "key") else {
        return;
    };
    let Some(value) = attr_value(event, "value") else {
        return;
    };

    let credential = credentials.entry(source.to_owned()).or_default();
    match key.as_str() {
        "Username" => credential.username = Some(value),
        "ClearTextPassword" => credential.password = Some(value),
        _ => {}
    }
}

fn auth_entry_from_source(
    source: &NugetConfigSource,
    credentials: &HashMap<String, NugetConfigCredentials>,
) -> Option<DotnetAuthEntry> {
    let credential = credentials.get(&source.name)?;
    let username = credential.username.as_deref()?;
    let password = credential.password.as_deref()?;
    let token = STANDARD.encode(format!("{username}:{password}"));

    Some(DotnetAuthEntry {
        registry: source.url.as_str().to_owned(),
        header_value: format!("Basic {token}"),
    })
}

fn source_from_add(event: &BytesStart<'_>) -> Option<NugetConfigSource> {
    Some(NugetConfigSource {
        name: attr_value(event, "key")?,
        url: attr_value(event, "value")?,
    })
}

fn source_mapping_from_package(
    source: &str,
    event: &BytesStart<'_>,
) -> Option<DotnetSourceMapping> {
    Some(DotnetSourceMapping {
        source: source.to_owned(),
        pattern: attr_value(event, "pattern")?,
    })
}

fn section_from_event(event: &BytesStart<'_>) -> NugetConfigSection {
    if event_name_is(event, "packageSources") {
        NugetConfigSection::PackageSources
    } else if event_name_is(event, "disabledPackageSources") {
        NugetConfigSection::DisabledPackageSources
    } else if event_name_is(event, "packageSourceCredentials") {
        NugetConfigSection::PackageSourceCredentials
    } else if event_name_is(event, "packageSourceMapping") {
        NugetConfigSection::PackageSourceMapping
    } else {
        NugetConfigSection::Other
    }
}

fn add_value_is_true(event: &BytesStart<'_>) -> bool {
    attr_value(event, "value").is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn attr_value(event: &BytesStart<'_>, name: &str) -> Option<String> {
    event.attributes().flatten().find_map(|attr| {
        (attr.key.as_ref() == name.as_bytes()).then(|| {
            std::str::from_utf8(attr.value.as_ref())
                .ok()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned)
        })?
    })
}

fn event_name(event: &BytesStart<'_>) -> Option<String> {
    std::str::from_utf8(event.name().as_ref())
        .ok()
        .map(ToOwned::to_owned)
}

fn event_name_is(event: &BytesStart<'_>, name: &str) -> bool {
    event.name().as_ref() == name.as_bytes()
}

fn is_remote_url(url: &str) -> bool {
    matches!(protocol_from_url(url).as_str(), "http:" | "https:")
}
