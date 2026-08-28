use super::{DocumentInput, standard_session};
use crate::RegistryResponseInput;
use versionlens_model::Ecosystem::Npm;

#[test]
fn npm_bare_relative_paths_are_invalid_versions() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("bare-relative-paths-are-invalid-versions.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "filepackage".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        )],
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
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("github-url-without-ref-is-fixed-git-dependency.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "owner/url".to_owned(),
            Npm,
            r#"[{"sha":"abcdef1234567890"}]"#.to_owned(),
        )],
    );

    crate::support::tests::assert_fixed_git_repository(&output);
}

#[test]
fn npm_package_manager_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("package-manager-dependencies-ignore-registry-updates.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "pnpm".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"10.34.4"}}"#.to_owned(),
        )],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("9.1.2"));
}

#[test]
fn npm_dev_engines_package_manager_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(
                "dev-engines-package-manager-dependencies-ignore-registry-updates.json",
            ),
            None,
        ),
        &[RegistryResponseInput::new(
            "npm".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"11.0.0"}}"#.to_owned(),
        )],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("^10.0.0"));
}

#[test]
fn npm_override_reference_dependencies_ignore_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("override-reference-dependencies-ignore-registry-updates.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "bar".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("override reference")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn npm_portal_dependencies_resolve_as_local_paths_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(
                "portal-dependencies-resolve-as-local-paths-without-registry-updates.json",
            ),
            None,
        ),
        &[RegistryResponseInput::new(
            "local".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        )],
    );

    crate::support::tests::assert_suggestion_without_edits(
        &output,
        0,
        "directoryNotFound",
        Some("../local"),
    );
}

#[test]
fn npm_exec_dependencies_are_not_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("exec-dependencies-are-not-registry-updates.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "generated".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        )],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "notSupported", None);
}

#[test]
fn npm_patch_dependencies_are_not_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/project/package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("patch-dependencies-are-not-registry-updates.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "@types/react".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"19.0.0"}}"#.to_owned(),
        )],
    );

    crate::support::tests::assert_suggestion_without_edits(&output, 0, "notSupported", None);
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/fixed/npm", name)
}
