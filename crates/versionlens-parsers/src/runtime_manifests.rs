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
mod tests {
use versionlens_model::{Ecosystem, VersionableKind};

use super::{
    parse_bun_version, parse_node_version, parse_nvmrc, parse_rust_toolchain,
    parse_rust_toolchain_toml,
};

fn fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/runtime_manifests/tests",
        name,
    )
}

fn assert_runtime(
    text: &str,
    dependencies: &[versionlens_model::Dependency],
    name: &str,
    requirement: &str,
    ecosystem: Ecosystem,
) {
    assert_eq!(dependencies.len(), 1);
    let dependency = &dependencies[0];
    assert_eq!(dependency.name, name);
    assert_eq!(dependency.requirement, requirement);
    assert_eq!(dependency.ecosystem, ecosystem);
    assert_eq!(dependency.group, "toolchain");
    assert_eq!(dependency.hosted_url.as_deref(), Some("toolchain"));
    assert_eq!(
        crate::document::test_support::extract_range(text, dependency.requirement_range),
        requirement
    );
}

#[test]
fn parses_nvmrc_alias_around_comments_and_reserved_settings() {
    let text = fixture("comments-and-lts.nvmrc");
    let dependencies = parse_nvmrc(text);

    assert_runtime(text, &dependencies, "node", "lts/*", Ecosystem::Npm);
}

#[test]
fn rejects_ambiguous_nvmrc_versions() {
    assert!(parse_nvmrc("20\n22\n").is_empty());
}

#[test]
fn parses_node_pin_and_bun_channel_without_normalizing_them() {
    let node = "  v20.11.1  \n";
    let node_dependencies = parse_node_version(node);
    assert_runtime(node, &node_dependencies, "node", "v20.11.1", Ecosystem::Npm);
    assert_eq!(
        node_dependencies[0].versionable_kind(),
        VersionableKind::Dependency
    );

    let bun = fixture("runtime.bun-version");
    let bun_dependencies = parse_bun_version(bun);
    assert_runtime(bun, &bun_dependencies, "bun", "canary", Ecosystem::Npm);
    assert_eq!(
        bun_dependencies[0].versionable_kind(),
        VersionableKind::Dependency
    );
}

#[test]
fn preserves_explicit_runtime_constraint_syntax() {
    let dependencies = parse_node_version(">=20");

    assert_eq!(dependencies[0].requirement, ">=20");
    assert_eq!(
        dependencies[0].versionable_kind(),
        VersionableKind::RuntimeConstraint
    );
}

#[test]
fn parses_legacy_and_toml_rust_toolchains_with_content_ranges() {
    let legacy = fixture("legacy.rust-toolchain");
    assert_runtime(
        legacy,
        &parse_rust_toolchain(legacy),
        "rust",
        "nightly-2026-09-01",
        Ecosystem::Cargo,
    );

    let toml = fixture("configured.rust-toolchain.toml");
    assert_runtime(
        toml,
        &parse_rust_toolchain_toml(toml),
        "rust",
        "1.90",
        Ecosystem::Cargo,
    );
    assert_runtime(
        toml,
        &parse_rust_toolchain(toml),
        "rust",
        "1.90",
        Ecosystem::Cargo,
    );
}

#[test]
fn escaped_toml_channel_replaces_the_raw_interior_only() {
    let text = fixture("escaped.rust-toolchain.toml");
    let dependencies = parse_rust_toolchain_toml(text);
    let dependency = &dependencies[0];
    let raw_channel = r"nightly\u002d2026\u002d09\u002d01";

    assert_eq!(dependency.requirement, "nightly-2026-09-01");
    assert_eq!(
        crate::document::test_support::extract_range(text, dependency.requirement_range),
        raw_channel
    );
    let start = text
        .find(raw_channel)
        .expect("raw channel should be present");
    let updated = format!(
        "{}stable{}",
        &text[..start],
        &text[start + raw_channel.len()..]
    );
    assert_eq!(
        updated,
        "[toolchain]\nchannel = \"stable\" # preserve this comment\n"
    );
}

#[test]
fn triple_quoted_toml_channel_uses_the_complete_interior() {
    let text = "[toolchain]\nchannel = '''stable''' # keep\n";
    let dependencies = parse_rust_toolchain_toml(text);

    assert_runtime(text, &dependencies, "rust", "stable", Ecosystem::Cargo);
}

#[test]
fn rejects_invalid_or_non_channel_toolchain_documents() {
    assert!(parse_rust_toolchain_toml("[toolchain]\npath = '/opt/rust'\n").is_empty());
    assert!(parse_rust_toolchain("[toolchain]\nchannel = 190\n").is_empty());
    assert!(parse_rust_toolchain("stablé\n").is_empty());
}

#[test]
fn document_dispatch_uses_each_runtime_manifest_parser() {
    let cases = [
        (
            "comments-and-lts.nvmrc",
            "file:///work/.nvmrc",
            "node",
            "lts/*",
        ),
        (
            "runtime.node-version",
            "file:///work/.node-version",
            "node",
            "v20.11.1",
        ),
        (
            "runtime.bun-version",
            "file:///work/.bun-version",
            "bun",
            "canary",
        ),
        (
            "legacy.rust-toolchain",
            "file:///work/rust-toolchain",
            "rust",
            "nightly-2026-09-01",
        ),
        (
            "configured.rust-toolchain.toml",
            "file:///work/rust-toolchain.toml",
            "rust",
            "1.90",
        ),
    ];

    for (fixture_name, uri, name, requirement) in cases {
        let dependencies = crate::parse_document(&crate::DocumentInput::new(
            uri,
            "plaintext",
            fixture(fixture_name),
            None,
        ));
        assert_eq!(dependencies.len(), 1, "{uri}");
        assert_eq!(dependencies[0].name, name, "{uri}");
        assert_eq!(dependencies[0].requirement, requirement, "{uri}");
    }
}
}
