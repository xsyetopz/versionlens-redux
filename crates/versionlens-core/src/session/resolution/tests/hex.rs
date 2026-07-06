use super::{DocumentInput, RegistryResponseInput, session_without_vulnerabilities};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_parsers::Ecosystem::Hex;

#[test]
fn resolves_mix_hex_alias_dependencies_against_target_package() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///mix.exs".to_owned(),
            language_id: "elixir".to_owned(),
            text: package_file_fixture(
                "resolves-mix-hex-alias-dependencies-against-target-package.exs",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "plug".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"2.0.0"},{"version":"1.20.0"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "plug");
    assert_eq!(
        output.suggestions[0].dependency.hosted_name.as_deref(),
        Some("plug_alias")
    );
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/hex")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
