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
