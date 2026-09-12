use std::ops::Range as ByteRange;

use toml_edit::Item;
use versionlens_model::{Dependency, Ecosystem};

use crate::{parse_toml_document, positions::offset_range};

struct RuntimeManifest<'a> {
    text: &'a str,
    name: &'static str,
    ecosystem: Ecosystem,
    requirement: &'a str,
    span: ByteRange<usize>,
}

pub(crate) fn parse_nvmrc(text: &str) -> Vec<Dependency> {
    let entries = text
        .split_inclusive('\n')
        .scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line.trim_end_matches(['\n', '\r'])))
        })
        .filter_map(nvmrc_entry)
        .collect::<Vec<_>>();
    let [(requirement, span)] = entries.as_slice() else {
        return vec![];
    };
    vec![runtime_dependency(RuntimeManifest {
        text,
        name: "node",
        ecosystem: Ecosystem::Npm,
        requirement,
        span: span.clone(),
    })]
}

pub(crate) fn parse_node_version(text: &str) -> Vec<Dependency> {
    parse_plain_runtime(text, "node", Ecosystem::Npm, false)
}

pub(crate) fn parse_bun_version(text: &str) -> Vec<Dependency> {
    parse_plain_runtime(text, "bun", Ecosystem::Npm, false)
}

pub(crate) fn parse_rust_toolchain(text: &str) -> Vec<Dependency> {
    let toml = parse_rust_toolchain_toml(text);
    if !toml.is_empty() || text.trim_start().starts_with('[') {
        toml
    } else {
        parse_plain_runtime(text, "rust", Ecosystem::Cargo, true)
    }
}

pub(crate) fn parse_rust_toolchain_toml(text: &str) -> Vec<Dependency> {
    let Ok(document) = parse_toml_document(text) else {
        return vec![];
    };
    let Some(channel) = document
        .get("toolchain")
        .and_then(Item::as_table)
        .and_then(|table| table.get("channel"))
        .and_then(Item::as_value)
    else {
        return vec![];
    };
    let Some(requirement) = channel.as_str().filter(|value| !value.is_empty()) else {
        return vec![];
    };
    let Some(span) = channel
        .span()
        .and_then(|span| literal_string_span(text, span))
    else {
        return vec![];
    };
    vec![runtime_dependency(RuntimeManifest {
        text,
        name: "rust",
        ecosystem: Ecosystem::Cargo,
        requirement,
        span,
    })]
}

fn nvmrc_entry(line: (usize, &str)) -> Option<(&str, ByteRange<usize>)> {
    let (line_start, line) = line;
    let value = line.split('#').next()?.trim();
    if value.is_empty() || value.contains('=') || value.contains(char::is_whitespace) {
        return None;
    }
    let start = line.find(value)?;
    Some((value, line_start + start..line_start + start + value.len()))
}

fn parse_plain_runtime(
    text: &str,
    name: &'static str,
    ecosystem: Ecosystem,
    ascii_only: bool,
) -> Vec<Dependency> {
    let requirement = text.trim();
    if requirement.is_empty()
        || requirement.contains(char::is_whitespace)
        || ascii_only && !requirement.is_ascii()
    {
        return vec![];
    }
    let Some(start) = text.find(requirement) else {
        return vec![];
    };
    vec![runtime_dependency(RuntimeManifest {
        text,
        name,
        ecosystem,
        requirement,
        span: start..start + requirement.len(),
    })]
}

fn literal_string_span(text: &str, span: ByteRange<usize>) -> Option<ByteRange<usize>> {
    let source = text.get(span.clone())?;
    let delimiter = ["\"\"\"", "'''", "\"", "'"].into_iter().find(|delimiter| {
        source.len() >= delimiter.len() * 2
            && source.starts_with(delimiter)
            && source.ends_with(delimiter)
    })?;
    Some(span.start + delimiter.len()..span.end - delimiter.len())
}

fn runtime_dependency(source: RuntimeManifest<'_>) -> Dependency {
    let range = offset_range(source.text, source.span.start, source.span.end);
    Dependency {
        name: source.name.to_owned(),
        requirement: source.requirement.to_owned(),
        ecosystem: source.ecosystem,
        group: "toolchain".to_owned(),
        hosted_url: Some("toolchain".to_owned()),
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

#[cfg(test)]
#[path = "runtime_manifests/tests.rs"]
mod tests;
