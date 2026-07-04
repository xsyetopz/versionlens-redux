use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};

#[test]
fn parses_cargo_toml_dependency_tables() {
    let text = r#"
[package]
version = "1.2.3"
edition = "2024"

[dependencies]
serde = "1.0"
local = { path = "../local" }
remote = { git = "https://example.test/repo.git" }

[dev-dependencies]
trybuild = "1.0"

[dev-dependencies.pretty_assertions]
version = "1.4"

[workspace.dependencies]
libc = "1.0"

[target.'cfg(unix)'.dependencies]
nix = "0.29"

[target.x86_64-pc-windows-msvc.dev-dependencies]
win-test = "0.1"
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Cargo);
    assert_eq!(dependencies[0].group, "package");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "serde");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "1.0"
    );
    assert_eq!(dependencies[2].requirement, "../local");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "../local"
    );
    assert_eq!(dependencies[3].requirement, "https://example.test/repo.git");
    assert_eq!(dependencies[4].group, "dev-dependencies");
    assert_eq!(dependencies[4].name, "trybuild");
    assert_eq!(dependencies[5].group, "dev-dependencies.pretty_assertions");
    assert_eq!(dependencies[5].name, "pretty_assertions");
    assert_eq!(dependencies[5].requirement, "1.4");
    assert_eq!(dependencies[6].group, "workspace.dependencies");
    assert_eq!(dependencies[6].name, "libc");
}

#[test]
fn parses_configured_cargo_target_dependency_tables() {
    let text = r#"
[dependencies]
serde = "1.0"

[target.'cfg(unix)'.dependencies]
nix = "0.29"

[target.x86_64-pc-windows-msvc.dev-dependencies]
win-test = "0.1"
"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &[
            "dependencies",
            "target.*.dependencies",
            "target.*.dev-dependencies",
        ],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "serde");
    assert_eq!(dependencies[1].group, "target.cfg(unix).dependencies");
    assert_eq!(dependencies[1].name, "nix");
    assert_eq!(dependencies[1].requirement, "0.29");
    assert_eq!(
        dependencies[2].group,
        "target.x86_64-pc-windows-msvc.dev-dependencies"
    );
    assert_eq!(dependencies[2].name, "win-test");
}

#[test]
fn parses_cargo_toml_nested_dependency_table_names() {
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[dependencies.serde]
version = "1.0"
features = ["derive"]

[workspace.dependencies.tokio]
version = "1"
"#
        .to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "dependencies.serde");
    assert_eq!(dependencies[0].name, "serde");
    assert_eq!(dependencies[0].requirement, "1.0");
    assert_eq!(dependencies[1].group, "workspace.dependencies.tokio");
    assert_eq!(dependencies[1].name, "tokio");
    assert_eq!(dependencies[1].requirement, "1");
}

#[test]
fn configured_cargo_suffix_wildcard_paths_match_deeper_tables() {
    let text = r#"
[workspace.metadata.tool.plugins.alpha]
version = "1.2.3"
"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["workspace.metadata.*"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].group,
        "workspace.metadata.tool.plugins.alpha"
    );
    assert_eq!(dependencies[0].name, "alpha");
    assert_eq!(dependencies[0].requirement, "1.2.3");
}

#[test]
fn parses_cargo_toml_renamed_package_dependency() {
    let text = r#"
[dependencies]
serde_json = { package = "serde-json", version = "1.0" }
private = { version = "2.0", registry = "private" }
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "serde-json");
    assert_eq!(dependencies[0].requirement, "1.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.0"
    );
    assert_eq!(dependencies[1].name, "private");
    assert_eq!(dependencies[1].requirement, "2.0");
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("private"));
}

#[test]
fn parses_smoke_cargo_smoke_shapes() {
    let text = r#"
[package]
name = "smoke"
version = "1.2.3"
description = "smoke test"
edition = "2024"

[dependencies]
backtrace = { version = "0.3.76", optional = true }

[dev-dependencies]
futures = { version = "0.3.32", default-features = false }
rustversion = "1.0.22"
syn = { version = "2.0.118", features = ["full"] }
thiserror = "2.0.18"
axum-extra = "0.12.6"
test = { path = "../.." }
smallvec = { git = "https://github.com/servo/rust-smallvec.git" }
libc = { workspace = true }

[dev-dependencies.trybuild]
version = "1.0.117"

[workspace.dependencies]
libc = "0.2.186"
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 11);
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[1].name, "backtrace");
    assert_eq!(dependencies[1].requirement, "0.3.76");
    assert_eq!(dependencies[2].name, "futures");
    assert_eq!(dependencies[2].requirement, "0.3.32");
    assert_eq!(dependencies[7].name, "test");
    assert_eq!(dependencies[7].requirement, "../..");
    assert_eq!(dependencies[8].name, "smallvec");
    assert_eq!(
        dependencies[8].requirement,
        "https://github.com/servo/rust-smallvec.git"
    );
    assert!(
        !dependencies
            .iter()
            .any(|dependency| dependency.group == "dev-dependencies" && dependency.name == "libc")
    );
    assert_eq!(dependencies[9].group, "dev-dependencies.trybuild");
    assert_eq!(dependencies[9].name, "trybuild");
    assert_eq!(dependencies[10].group, "workspace.dependencies");
    assert_eq!(dependencies[10].name, "libc");
    assert_eq!(dependencies[10].requirement, "0.2.186");
}
