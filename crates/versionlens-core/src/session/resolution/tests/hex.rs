use super::{DocumentInput, session_without_vulnerabilities};
use crate::RegistryResponseInput;
use versionlens_model::Ecosystem::Hex;

#[test]
fn resolves_mix_hex_alias_dependencies_against_target_package() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///mix.exs".to_owned(),
            "elixir".to_owned(),
            package_file_fixture("resolves-mix-hex-alias-dependencies-against-target-package.exs"),
            None,
        ),
        &[RegistryResponseInput::new(
            "plug".to_owned(),
            Hex,
            r#"{"releases":[{"version":"2.0.0"},{"version":"1.20.0"}]}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].dependency.name, "plug");
    assert_eq!(
        output.suggestions[0].dependency.hosted_name.as_deref(),
        Some("plug_alias")
    );
    assert_update(&output, "2.0.0");
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/hex", name)
}
use super::assert_update;
