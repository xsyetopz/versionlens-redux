use self::NugetConfigSection::{
    DisabledPackageSources as NugetDisabledPackageSources, Other as NugetOther,
    PackageSourceCredentials as NugetPackageSourceCredentials,
    PackageSourceMapping as NugetPackageSourceMapping, PackageSources as NugetPackageSources,
};
use quick_xml::events::Event::{
    Empty as XmlEventEmpty, End as XmlEventEnd, Eof as XmlEventEof, Start as XmlEventStart,
};
use std::collections::{HashMap, HashSet};

use base64::{Engine, engine::general_purpose::STANDARD};
use quick_xml::events::BytesStart;

type NugetXmlEvent<'a> = BytesStart<'a>;
type NugetDisabledSources = HashSet<String>;
type NugetCredentialsBySource = HashMap<String, NugetConfigCredentials>;

use super::protocol::protocol_from_url;
use super::schema::{DotnetAuthEntry, DotnetNamedSource, DotnetNugetConfig, DotnetSourceMapping};
use std::str;

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
    disabled: NugetDisabledSources,
    credentials: NugetCredentialsBySource,
    source_mappings: Vec<DotnetSourceMapping>,
    changes: NugetConfigChanges,
}

#[derive(Debug, Default)]
struct NugetConfigChanges {
    removed_sources: NugetDisabledSources,
    removed_source_mappings: NugetDisabledSources,
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
    let mut reader = crate::xml_reader(text);
    let mut state = crate::default();

    loop {
        match reader.read_event() {
            Ok(XmlEventStart(event)) => start_tag(&mut state, &event),
            Ok(XmlEventEnd(event)) => end_tag(&mut state, event.name().as_ref()),
            Ok(XmlEventEmpty(event)) => collect_empty_tag(&mut state, &event),
            Ok(XmlEventEof) => break,
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
    disabled: &NugetDisabledSources,
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
    disabled: &NugetDisabledSources,
    credentials: &NugetCredentialsBySource,
) -> Vec<DotnetAuthEntry> {
    sources
        .iter()
        .filter(|source| !disabled.contains(&source.name))
        .filter(|source| is_remote_url(&source.url))
        .filter_map(|source| auth_entry_from_source(source, credentials))
        .collect()
}

fn start_tag(state: &mut NugetConfigState, event: &NugetXmlEvent<'_>) {
    match section_from_event(event) {
        NugetOther if state.section == NugetPackageSourceCredentials => {
            state.credential_source = event_name(event);
        }
        NugetOther
            if state.section == NugetPackageSourceMapping
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

    if name == b"packageSource" && state.section == NugetPackageSourceMapping {
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
        state.section = NugetOther;
    }
}

fn collect_empty_tag(state: &mut NugetConfigState, event: &NugetXmlEvent<'_>) {
    if event_name_is(event, "clear") {
        clear_section(state);
        return;
    }
    if event_name_is(event, "remove") {
        remove_section_entry(state, event);
        return;
    }

    match state.section {
        NugetPackageSources => {
            if event_name_is(event, "add")
                && let Some(source) = source_from_add(event)
            {
                state.sources.push(source);
            }
        }
        NugetDisabledPackageSources => {
            if event_name_is(event, "add")
                && add_value_is_true(event)
                && let Some(name) = attr_value(event, "key")
            {
                state.disabled.insert(name);
            }
        }
        NugetPackageSourceCredentials => {
            if event_name_is(event, "add")
                && let Some(source) = &state.credential_source
            {
                collect_credential(&mut state.credentials, source, event);
            }
        }
        NugetPackageSourceMapping => {
            if event_name_is(event, "package")
                && let Some(source) = &state.mapping_source
                && let Some(mapping) = source_mapping_from_package(source, event)
            {
                state.source_mappings.push(mapping);
            }
        }
        NugetOther => {}
    }
}

fn remove_section_entry(state: &mut NugetConfigState, event: &NugetXmlEvent<'_>) {
    let Some(key) = attr_value(event, "key") else {
        return;
    };

    match state.section {
        NugetPackageSources => {
            state.sources.retain(|source| source.name != key);
            state.changes.removed_sources.insert(key);
        }
        NugetDisabledPackageSources => {
            state.disabled.remove(&key);
        }
        NugetPackageSourceCredentials => {
            if let Some(source) = &state.credential_source
                && let Some(credentials) = state.credentials.get_mut(source)
            {
                remove_credential(credentials, &key);
            }
        }
        NugetPackageSourceMapping => {
            state
                .source_mappings
                .retain(|mapping| mapping.source != key);
            state.changes.removed_source_mappings.insert(key);
        }
        NugetOther => {}
    }
}

fn clear_section(state: &mut NugetConfigState) {
    match state.section {
        NugetPackageSources => {
            state.sources.clear();
            state.changes.clear_sources = true;
        }
        NugetDisabledPackageSources => state.disabled.clear(),
        NugetPackageSourceCredentials => {
            state.credentials.clear();
            state.changes.clear_auth_entries = true;
        }
        NugetPackageSourceMapping => {
            state.source_mappings.clear();
            state.changes.clear_source_mappings = true;
        }
        NugetOther => {}
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
    credentials: &mut NugetCredentialsBySource,
    source: &str,
    event: &NugetXmlEvent<'_>,
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
    credentials: &NugetCredentialsBySource,
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

fn source_from_add(event: &NugetXmlEvent<'_>) -> Option<NugetConfigSource> {
    Some(NugetConfigSource {
        name: attr_value(event, "key")?,
        url: attr_value(event, "value")?,
    })
}

fn source_mapping_from_package(
    source: &str,
    event: &NugetXmlEvent<'_>,
) -> Option<DotnetSourceMapping> {
    Some(DotnetSourceMapping {
        source: source.to_owned(),
        pattern: attr_value(event, "pattern")?,
    })
}

fn section_from_event(event: &NugetXmlEvent<'_>) -> NugetConfigSection {
    if event_name_is(event, "packageSources") {
        NugetPackageSources
    } else if event_name_is(event, "disabledPackageSources") {
        NugetDisabledPackageSources
    } else if event_name_is(event, "packageSourceCredentials") {
        NugetPackageSourceCredentials
    } else if event_name_is(event, "packageSourceMapping") {
        NugetPackageSourceMapping
    } else {
        NugetOther
    }
}

fn add_value_is_true(event: &NugetXmlEvent<'_>) -> bool {
    attr_value(event, "value").is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

fn attr_value(event: &NugetXmlEvent<'_>, name: &str) -> Option<String> {
    event.attributes().flatten().find_map(|attr| {
        (attr.key.as_ref() == name.as_bytes()).then(|| {
            str::from_utf8(attr.value.as_ref())
                .ok()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| value.to_owned())
        })?
    })
}

fn event_name(event: &NugetXmlEvent<'_>) -> Option<String> {
    str::from_utf8(event.name().as_ref())
        .ok()
        .map(|value| value.to_owned())
}

fn event_name_is(event: &NugetXmlEvent<'_>, name: &str) -> bool {
    event.name().as_ref() == name.as_bytes()
}

fn is_remote_url(url: &str) -> bool {
    matches!(protocol_from_url(url).as_str(), "http:" | "https:")
}
