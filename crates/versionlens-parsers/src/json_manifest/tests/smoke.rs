use super::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::{Dub, Npm};

#[test]
fn parses_smoke_pnpm_package_json_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-pnpm-package-json-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "packageManager");
    assert_eq!(dependencies[0].name, "pnpm");
    assert_eq!(dependencies[0].requirement, "10.34.4");
    assert_eq!(
        dependencies[1].requirement,
        "file:../overrides/package.json"
    );
    assert_eq!(dependencies[2].name, "types-react");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(dependencies[2].requirement_prefix, "npm:types-react@");
    assert_eq!(dependencies[3].group, "pnpm.overrides");
    assert_eq!(dependencies[4].name, "axios");
    assert_eq!(dependencies[5].name, "typescript");
}

#[test]
fn parses_smoke_npm_custom_file_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-npm-custom-file-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/web-module.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "devDependencies");
    assert_eq!(dependencies[0].name, "typescript");
    assert_eq!(dependencies[0].requirement, "^6.0.3");
}

#[test]
fn parses_smoke_npm_git_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-npm-git-smoke-shapes.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "devDependencies");
    assert_eq!(dependencies[0].name, "gitpkgnotfound1");
    assert_eq!(
        dependencies[0].requirement,
        "git+https://git@github.com/testuser/test.git"
    );
    assert_eq!(dependencies[1].name, "gitpkgnotfound2");
    assert_eq!(
        dependencies[1].requirement,
        "git+ssh://git@some.com/testuser/test.git"
    );
}

#[test]
fn parses_smoke_npm_faq_and_npmrc_smoke_shapes() {
    let faq = r#"{
  "devDependencies": {
    "projectz": "4.2.0",
    "@types/node": "26.0.1",
    "typescript": "^6.0.3"
  }
}"#;
    let faq_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: faq.to_owned(),
        workspace_root: None,
    });

    assert_eq!(faq_dependencies.len(), 3);
    assert_eq!(faq_dependencies[0].ecosystem, Npm);
    assert_eq!(faq_dependencies[0].group, "devDependencies");
    assert_eq!(faq_dependencies[0].name, "projectz");
    assert_eq!(faq_dependencies[0].requirement, "4.2.0");
    assert_eq!(faq_dependencies[1].name, "@types/node");
    assert_eq!(faq_dependencies[1].requirement, "26.0.1");
    assert_eq!(faq_dependencies[2].name, "typescript");
    assert_eq!(faq_dependencies[2].requirement, "^6.0.3");

    let npmrc = r#"{
  "name": "smoke-test",
  "title": "smoke test",
  "dependencies": {},
  "devDependencies": {
    "@scope/some-package": "0.1"
  }
}"#;
    let npmrc_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: npmrc.to_owned(),
        workspace_root: None,
    });

    assert_eq!(npmrc_dependencies.len(), 1);
    assert_eq!(npmrc_dependencies[0].ecosystem, Npm);
    assert_eq!(npmrc_dependencies[0].group, "devDependencies");
    assert_eq!(npmrc_dependencies[0].name, "@scope/some-package");
    assert_eq!(npmrc_dependencies[0].requirement, "0.1");
}

#[test]
fn parses_smoke_dub_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dub-smoke-shapes.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "~>3.11.0");
    assert_eq!(dependencies[4].name, "painlessjson");
    assert_eq!(dependencies[4].requirement, "1.4.0");
}

#[test]
fn parses_smoke_dub_selections_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dub-selections-smoke-shapes.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dub.selections.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Dub);
    assert_eq!(dependencies[0].group, "versions");
    assert_eq!(dependencies[0].name, "gtk-d:gtkd");
    assert_eq!(dependencies[0].requirement, "3.11.0");
    assert_eq!(dependencies[2].name, "derelict-sdl2");
    assert_eq!(dependencies[2].requirement, "2.1.4");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/smoke")
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
