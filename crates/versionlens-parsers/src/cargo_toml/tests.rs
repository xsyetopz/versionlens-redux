use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Cargo;

#[test]
fn parses_cargo_toml_dependency_tables() {
    let text = package_file_fixture("parses-cargo-toml-dependency-tables.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 9);
    assert_eq!(dependencies[0].ecosystem, Cargo);
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
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("path"));
    assert_eq!(dependencies[3].requirement, "https://example.test/repo.git");
    assert_eq!(dependencies[4].group, "dev-dependencies");
    assert_eq!(dependencies[4].name, "trybuild");
    assert_eq!(dependencies[5].group, "dev-dependencies.pretty_assertions");
    assert_eq!(dependencies[5].name, "pretty_assertions");
    assert_eq!(dependencies[5].requirement, "1.4");
    assert_eq!(dependencies[6].group, "workspace.dependencies");
    assert_eq!(dependencies[6].name, "libc");
    assert_eq!(dependencies[7].group, "target.cfg(unix).dependencies");
    assert_eq!(dependencies[7].name, "nix");
    assert_eq!(dependencies[7].requirement, "0.29");
    assert_eq!(
        dependencies[8].group,
        "target.x86_64-pc-windows-msvc.dev-dependencies"
    );
    assert_eq!(dependencies[8].name, "win-test");
    assert_eq!(dependencies[8].requirement, "0.1");
}

#[test]
fn cargo_path_dependencies_prefer_path_over_version() {
    let text = r#"
[dependencies]
local = { version = "1.2.3", path = "crates/local" }
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "local");
    assert_eq!(dependencies[0].requirement, "crates/local");
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("path"));
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "crates/local"
    );
}

#[test]
fn parses_configured_cargo_target_dependency_tables() {
    let text = package_file_fixture("parses-configured-cargo-target-dependency-tables.txt");
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
        text: package_file_fixture("parses-cargo-toml-nested-dependency-table-names.toml")
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
fn parses_cargo_target_dependency_tables_by_default() {
    let text = package_file_fixture("parses-cargo-target-dependency-tables-by-default.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "target.cfg(unix).dependencies");
    assert_eq!(dependencies[0].name, "nix");
    assert_eq!(dependencies[0].requirement, "0.29");
    assert_eq!(
        dependencies[1].group,
        "target.x86_64-pc-windows-msvc.dev-dependencies"
    );
    assert_eq!(dependencies[1].name, "win-test");
    assert_eq!(dependencies[1].requirement, "0.1");
    assert_eq!(
        dependencies[2].group,
        "target.cfg(target_arch = \"wasm32\").build-dependencies"
    );
    assert_eq!(dependencies[2].name, "wasm-build");
    assert_eq!(dependencies[2].requirement, "1.2");
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("private"));
}

#[test]
fn parses_cargo_workspace_inherited_dependencies() {
    let text = package_file_fixture("parses-cargo-workspace-inherited-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/member/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: Some("/work".to_owned()),
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "regex");
    assert_eq!(dependencies[0].requirement, "workspace:true");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "true"
    );
    assert_eq!(dependencies[1].group, "build-dependencies");
    assert_eq!(dependencies[1].name, "cc");
    assert_eq!(dependencies[1].requirement, "workspace:true");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "true"
    );
    assert_eq!(dependencies[2].group, "target.cfg(unix).dev-dependencies");
    assert_eq!(dependencies[2].name, "rand");
    assert_eq!(dependencies[2].requirement, "workspace:true");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "true"
    );
}

#[test]
fn configured_cargo_suffix_wildcard_paths_match_deeper_tables() {
    let text =
        package_file_fixture("configured-cargo-suffix-wildcard-paths-match-deeper-tables.txt");
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
    let text = package_file_fixture("parses-cargo-toml-renamed-package-dependency.toml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "serde_json");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("serde-json"));
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
    let text = package_file_fixture("parses-smoke-cargo-smoke-shapes.toml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Cargo.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
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
    assert_eq!(dependencies[9].group, "dev-dependencies");
    assert_eq!(dependencies[9].name, "libc");
    assert_eq!(dependencies[9].requirement, "workspace:true");
    assert_eq!(dependencies[10].group, "dev-dependencies.trybuild");
    assert_eq!(dependencies[10].name, "trybuild");
    assert_eq!(dependencies[11].group, "workspace.dependencies");
    assert_eq!(dependencies[11].name, "libc");
    assert_eq!(dependencies[11].requirement, "0.2.186");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/cargo_toml/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
