use super::{
    ApplyCommandRequest, DocumentInput, session_with_vulnerability_visibility, standard_session,
};
use crate::RegistryResponseInput;
use crate::contract::ResolveDocumentOutput;
use versionlens_model::Ecosystem::*;

include!("update/platforms.rs");
include!("update/security.rs");
include!("update/selection.rs");
fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/commands/update", name)
}

fn assert_single_edit(output: &ResolveDocumentOutput, expected: &str) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, expected);
}

fn assert_single_dependency_update(
    output: &ResolveDocumentOutput,
    group: &str,
    current: &str,
    selected: &str,
) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, group);
    assert_eq!(output.suggestions[0].dependency.requirement, current);
    assert_single_edit(output, selected);
}

fn assert_project_update(output: &ResolveDocumentOutput, name: &str, expected: &str) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, name);
    assert_single_edit(output, expected);
}

fn assert_project_edit(output: &ResolveDocumentOutput, expected: &str) {
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_single_edit(output, expected);
}

include!("update/project.rs");
include!("update/jvm.rs");
include!("update/languages.rs");
include!("update/packages.rs");
include!("update/toolchains.rs");
