use super::{DocumentInput, session_with_dependency_properties, standard_session};
use crate::VersionLensSession;
use crate::contract::ResolveDocumentOutput;
use versionlens_model::Ecosystem::*;
use versionlens_model::TextEdit;

fn sort_fixture(
    session: &VersionLensSession,
    uri: &str,
    language: &str,
    fixture: &str,
) -> ResolveDocumentOutput {
    session.apply_command(
        DocumentInput::new(
            uri.to_owned(),
            language.to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some("sort"),
        None,
        &[],
    )
}

fn assert_sort_output(output: &ResolveDocumentOutput, expected: &[&str]) {
    assert!(output.suggestions.is_empty());
    if expected.is_empty() {
        assert!(output.edits.is_empty());
        return;
    }
    assert_eq!(output.edits.len(), expected.len());
    for (edit, expected_text) in output.edits.iter().zip(expected) {
        assert_eq!(edit.new_text, *expected_text);
    }
}

include!("sort/manifests.rs");
include!("sort/ecosystems.rs");
fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/commands/sort", name)
}

include!("sort/exclusions.rs");
