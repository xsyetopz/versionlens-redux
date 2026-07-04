use super::{DocumentInput, Ecosystem, RegistryResponseInput, standard_session};

#[test]
fn npm_bare_relative_paths_are_invalid_versions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"filepackage":"../../.."}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "filepackage".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "error");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("invalid version")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_github_url_without_ref_is_fixed_git_dependency() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"git-url":"git+https://github.com/owner/url.git"}}"#
                .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/url".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"[{"sha":"abcdef1234567890"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_package_manager_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"packageManager":"pnpm@9.1.2"}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "pnpm".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"10.34.4"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("9.1.2"));
    assert!(output.edits.is_empty());
}
