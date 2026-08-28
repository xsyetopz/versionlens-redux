use crate::RegistryResponseInput;
use versionlens_model::DocumentInput;

use versionlens_model::Ecosystem::Dub;

#[test]
fn resolves_repo_versions_as_current_when_registry_matches() {
    let session = crate::support::tests::test_session(true);

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///dub.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("resolves-repo-versions-as-current-when-registry-matches.json"),
            None,
        ),
        &[RegistryResponseInput::new(
            "vibe-d".to_owned(),
            Dub,
            r#"{"versions":[{"version":"~master"},{"version":"0.9.0"},{"version":"0.8.0"}]}"#
                .to_owned(),
        )],
    );

    assert_eq!(output.suggestions[0].latest.as_deref(), Some("~master"));
    assert_eq!(output.suggestions[0].status, "current");
    assert!(output.edits.is_empty());
}

#[test]
fn pub_hosted_git_dependencies_are_fixed_without_git_suffix() {
    let session = crate::support::tests::test_session(false);
    let output = session.resolve_document(DocumentInput::new(
        "file:///pubspec.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("pub-hosted-git-dependencies-are-fixed-without-git-suffix.yaml"),
        None,
    ));

    assert_eq!(output.suggestions[0].dependency.name, "repo");
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
}

#[test]
fn pub_git_tag_pattern_dependencies_are_fixed_without_pub_registry_lookup() {
    let session = crate::support::tests::test_session(false);
    let output = session.resolve_document(DocumentInput::new(
        "file:///pubspec.yaml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture(
            "pub-git-tag-pattern-dependencies-are-fixed-without-pub-registry-lookup.yaml",
        ),
        None,
    ));

    assert_eq!(output.suggestions[0].dependency.name, "kittens");
    assert_eq!(
        output.suggestions[0].dependency.requirement,
        "git@github.com:munificent/kittens.git"
    );
    crate::support::tests::assert_fixed_git_repository(&output);
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/dependency/tests", name)
}
