use super::{ApplyCommandRequest, DocumentInput, RegistryResponseInput, standard_session};
use versionlens_model::Ecosystem::{GitHub, Npm};

#[test]
fn apply_command_updates_only_selected_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        package_input(r#"{"dependencies":{"left-pad":"1.0.0","is-odd":"2.0.0"}}"#),
        None,
        Some("left-pad"),
        &[
            npm_response("left-pad", "1.1.0"),
            npm_response("is-odd", "3.0.0"),
        ],
    );

    super::assert_single_named_edit(&output, "left-pad", "1.1.0");
}

#[test]
fn apply_command_rejects_unknown_commands_without_resolving() {
    let session = standard_session();
    let input = package_input(r#"{"dependencies":{"left-pad":"1.0.0"}}"#);

    let output = session.apply_command(
        input.clone(),
        Some("executeArbitraryCommand"),
        Some("left-pad"),
        &[npm_response("left-pad", "9.9.9")],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
    assert!(session.analyze_document(input).code_lenses.is_empty());
}

#[test]
fn selected_version_without_update_command_is_ignored() {
    let session = standard_session();
    let response = RegistryResponseInput {
        package: "left-pad".to_owned(),
        ecosystem: Npm,
        body: r#"{"dist-tags":{"latest":"1.0.0+build.2"},"versions":{"1.0.0+build.2":{}}}"#
            .to_owned(),
    };

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: package_input(r#"{"dependencies":{"left-pad":"1.0.0+build.1"}}"#),
        command: None,
        dependency_name: Some("left-pad"),
        selected_version: Some("999.0.0"),
        responses: &[response],
    });

    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions.len(), 1);
    assert_ne!(output.suggestions[0].latest.as_deref(), Some("999.0.0"));
}

#[test]
fn selected_older_github_release_applies_as_a_downgrade() {
    let session = standard_session();
    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            "steps:\n  - uses: actions/checkout@v7.0.1\n".to_owned(),
            None,
        ),
        command: Some("update"),
        dependency_name: Some("actions/checkout"),
        selected_version: Some("6.0.0"),
        responses: &[RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            GitHub,
            r#"[{"name":"v7.0.1"},{"name":"v7.0.0"},{"name":"v6.0.0"}]"#.to_owned(),
        )],
    });

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "v6.0.0");
}

fn package_input(text: &str) -> DocumentInput {
    DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
        version: None,
    }
}

fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
    RegistryResponseInput {
        package: package.to_owned(),
        ecosystem: Npm,
        body: format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
    }
}
