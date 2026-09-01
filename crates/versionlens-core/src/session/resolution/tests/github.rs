use super::*;

fn resolve_github_fixture(
    fixture: &str,
    package: &str,
    ecosystem: Ecosystem,
    body: &str,
) -> ResolveDocumentOutput {
    let session = standard_session();
    let (uri, language) = if fixture.ends_with("Gemfile") {
        ("file:///Gemfile", "ruby")
    } else {
        ("file:///package.json", "json")
    };
    session.resolve_document_with_responses(
        DocumentInput::new(
            uri.to_owned(),
            language.to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        &[RegistryResponseInput::new(
            package.to_owned(),
            ecosystem,
            body.to_owned(),
        )],
    )
}

fn resolve_checkout_tag(body: &str) -> ResolveDocumentOutput {
    standard_session().resolve_document_with_responses(
        DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            "steps:\n  - uses: actions/checkout@v4\n".to_owned(),
            None,
        ),
        &[RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            GitHub,
            body.to_owned(),
        )],
    )
}

#[test]
fn resolves_github_action_tag_references_with_incremental_choices() {
    let output = resolve_checkout_tag(&github_action_tags());

    assert_update(&output, "v4.2.0");
}

#[test]
fn resolves_major_action_refs_when_registry_has_only_concrete_release_tags() {
    let output = resolve_checkout_tag(r#"[{"name":"v4.2.0"},{"name":"v4.1.0"},{"name":"v4.0.0"}]"#);

    assert_update(&output, "v4.2.0");
}

#[test]
fn does_not_resolve_major_action_ref_from_a_mismatched_major() {
    let session = standard_session();
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            "steps:\n  - uses: actions/checkout@v3\n".to_owned(),
            None,
        ),
        &[RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            GitHub,
            r#"[{"name":"v4.2.0"},{"name":"v4.1.0"}]"#.to_owned(),
        )],
    );

    assert!(output.edits.is_empty());
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status != "updateAvailable")
    );
}

#[test]
fn does_not_suggest_for_numeric_or_dotted_refs_without_a_matching_tag() {
    for requirement in ["2024", "2024.01", "${{ matrix.action_ref }}", "main"] {
        let session = standard_session();
        let output = session.resolve_document_with_responses(
            DocumentInput::new(
                "file:///work/.github/workflows/ci.yml".to_owned(),
                "yaml".to_owned(),
                format!("steps:\n  - uses: actions/checkout@{requirement}\n"),
                None,
            ),
            &[RegistryResponseInput::new(
                "actions/checkout".to_owned(),
                GitHub,
                github_action_tags(),
            )],
        );

        assert!(output.edits.is_empty(), "unexpected edit for {requirement}");
        assert!(
            output
                .suggestions
                .iter()
                .all(|suggestion| suggestion.status != "updateAvailable")
        );
    }
}

fn github_action_tags() -> String {
    r#"[{"name":"v4.2.0"},{"name":"v4.1.0"},{"name":"v4.0.0"},{"name":"v4"}]"#.to_owned()
}

#[test]
fn resolves_reusable_workflow_tags_using_repository_identity() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///work/.github/workflows/release.yml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("reusable-workflow-tags.yaml"),
        None,
    );
    let dependencies = parse_document(&input);
    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].hosted_name.as_deref(),
        Some("acme/automation")
    );
    let response = RegistryResponseInput::new(
        "acme/automation".to_owned(),
        GitHub,
        r#"[{"name":"v1.3.0"},{"name":"v1"}]"#.to_owned(),
    );
    assert_eq!(
        session.latest_from_responses(&dependencies[0], std::slice::from_ref(&response)),
        Some("v1.3.0".to_owned())
    );

    let output = session.resolve_document_with_responses(input, std::slice::from_ref(&response));

    assert_update(&output, "v1.3.0");
}

#[test]
fn resolves_npm_github_dependencies_from_tags() {
    let output = resolve_github_fixture(
        "npm-github-dependencies-from-tags.json",
        "octokit/core.js",
        Npm,
        r#"[{"name":"v2.5.0"},{"name":"v1.9.0"}]"#,
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(
        output.edits[0].new_text,
        "github:octokit/core.js#semver:2.5.0"
    );
}

#[test]
fn resolves_npm_github_commit_dependencies_from_commits() {
    let output = resolve_github_fixture(
        "npm-github-commit-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_update(&output, "github:owner/commit#abcdef1");
}

#[test]
fn resolves_npm_github_url_commit_dependencies_from_commits() {
    let output = resolve_github_fixture(
        "npm-github-url-commit-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_update(&output, "git+https://github.com/owner/commit.git#abcdef1");
}

#[test]
fn resolves_npm_github_git_ssh_dependencies_from_commits() {
    let output = resolve_github_fixture(
        "npm-github-git-ssh-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_update(&output, "git+ssh://git@github.com/owner/commit.git#abcdef1");
}

#[test]
fn resolves_npm_github_git_ssh_colon_dependencies_from_commits() {
    let output = resolve_github_fixture(
        "npm-github-git-ssh-colon-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_update(&output, "git+ssh://git@github.com:owner/commit.git#abcdef1");
}

#[test]
fn resolves_npm_github_dependencies_without_refs_from_commits() {
    let output = resolve_github_fixture(
        "npm-github-dependencies-without-refs-from-commits.json",
        "owner/bare",
        Npm,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_update(&output, "github:owner/bare#abcdef1");
}

#[test]
fn routes_npm_github_tag_dependencies_to_tags() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("npm-github-tag-dependencies-to-tags.json"),
        None,
    ));

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/octokit/core.js/tags"]
    );
}

#[test]
fn routes_npm_github_dependencies_without_refs_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("npm-github-dependencies-without-refs-to-commits.json"),
        None,
    ));

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/owner/bare/commits"]
    );
}

#[test]
fn routes_npm_github_commit_dependencies_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("npm-github-commit-dependencies-to-commits.json"),
        None,
    ));

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/owner/commit/commits"]
    );
}

#[test]
fn resolves_ruby_github_tag_dependencies_from_tags() {
    let output = resolve_github_fixture(
        "ruby-github-tag-dependencies-from-tagsGemfile",
        "rspec/rspec-rails",
        Ruby,
        r#"[{"name":"v6.1.0"},{"name":"v6.0.1"}]"#,
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"tag: "v6.1.0""#);
}

#[test]
fn resolves_ruby_github_dependencies_without_ref_from_commits() {
    let output = resolve_github_fixture(
        "ruby-github-dependencies-without-ref-from-commitsGemfile",
        "heartcombo/devise",
        Ruby,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_commit_ref_update(&output);
}

#[test]
fn resolves_ruby_github_ref_dependencies_from_commits() {
    let output = resolve_github_fixture(
        "ruby-github-ref-dependencies-from-commitsGemfile",
        "rspec/rspec-core",
        Ruby,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"ref: "abcdef1""#);
}

#[test]
fn resolves_ruby_git_github_tag_dependencies_from_tags() {
    let output = resolve_github_fixture(
        "ruby-git-github-tag-dependencies-from-tagsGemfile",
        "rails/rails",
        Ruby,
        r#"[{"name":"v8.0.0"},{"name":"v7.0.0"}]"#,
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"tag: "v8.0.0""#);
}

#[test]
fn resolves_ruby_git_github_dependencies_without_ref_from_commits() {
    let output = resolve_github_fixture(
        "ruby-git-github-dependencies-without-ref-from-commitsGemfile",
        "rails/rails",
        Ruby,
        r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
    );

    assert_commit_ref_update(&output);
}

#[test]
fn routes_ruby_github_ref_dependencies_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput::new(
        "file:///Gemfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("ruby-github-ref-dependencies-to-commitsGemfile"),
        None,
    ));

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/rspec/rspec-core/commits"]
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/github", name)
}

fn assert_commit_ref_update(output: &crate::contract::ResolveDocumentOutput) {
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#", ref: "abcdef1""#);
}
use super::assert_update;
