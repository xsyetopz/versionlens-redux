use super::*;
use std::io::{Read, Write};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};

fn github_api_server(responses: Vec<(u16, &'static str)>) -> (String, JoinHandle<Vec<String>>) {
    let listener = crate::support::tests::tcp_listener_bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let base_url = format!("http://{}/repos/", listener.local_addr().unwrap());
    let server = spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut paths = vec![];
        for (status, body) in responses {
            let mut accepted = None;
            while Instant::now() < deadline {
                if let Ok(connection) = listener.accept() {
                    accepted = Some(connection);
                    break;
                }
                sleep(Duration::from_millis(5));
            }
            let Some((mut stream, _)) = accepted else {
                break;
            };
            let mut request = [0_u8; 2048];
            let length = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..length]);
            let path = request
                .lines()
                .next()
                .and_then(|line| line.split_whitespace().nth(1))
                .unwrap()
                .to_owned();
            let reason = if status == 200 { "OK" } else { "Not Found" };
            let response = format!(
                "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            paths.push(path);
        }
        paths
    });
    (base_url, server)
}

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

fn resolve_action_source(source: &str, body: &str) -> ResolveDocumentOutput {
    standard_session().resolve_document_with_responses(
        DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            source.to_owned(),
            None,
        ),
        &[RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            GitHub,
            body.to_owned(),
        )],
    )
}

fn action_choices(
    source: &str,
    latest: &str,
    body: &str,
) -> Vec<versionlens_suggestions::UpdateChoice> {
    let input = DocumentInput::new(
        "file:///work/.github/workflows/ci.yml".to_owned(),
        "yaml".to_owned(),
        source.to_owned(),
        None,
    );
    let dependency = parse_document(&input).into_iter().next().unwrap();
    crate::fetch::response_update_choices(&dependency, latest, body, true, &[])
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
fn sha_pinned_action_at_latest_has_no_redundant_latest_choice() {
    let sha = "3d3c42e5aac5ba805825da76410c181273ba90b1";
    let source = format!("steps:\n  - uses: actions/checkout@{sha} # v7.0.1\n");
    let output = resolve_action_source(
        &source,
        &format!(
            r#"[{{"name":"v7.0.1","commit":{{"sha":"{sha}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
        ),
    );

    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "current");
    let choices = action_choices(
        &source,
        "7.0.1",
        &format!(
            r#"[{{"name":"v7.0.1","commit":{{"sha":"{sha}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
        ),
    );
    assert!(
        choices
            .iter()
            .all(|choice| !choice.label.starts_with("latest"))
    );
    assert!(
        choices
            .iter()
            .any(|choice| choice.label == "downgrade" && choice.version == "6.0.0")
    );
}

#[test]
fn concrete_azure_login_release_pin_resolves_as_current() {
    let sha = "7ddb5af1ef8758cf1353cf3b42f940aee27ba21c";
    let source = format!("steps:\n  - uses: azure/login@{sha} # v3.0.2\n");
    let output = standard_session().resolve_document_with_responses(
        DocumentInput::new(
            "file:///work/.github/workflows/publish.yml".to_owned(),
            "yaml".to_owned(),
            source,
            None,
        ),
        &[RegistryResponseInput::new(
            "azure/login".to_owned(),
            GitHub,
            format!(
                r#"[{{"name":"v3.0.2","commit":{{"sha":"{sha}"}}}},{{"name":"v3.0.1","commit":{{"sha":"f5d393ae46f8fde4be8b75f32e3fc50e654ad0ca"}}}}]"#
            ),
        )],
    );

    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("3.0.2"));
}

#[test]
fn annotated_azure_login_tag_object_pin_is_resolved_through_the_exact_ref() {
    let (base_url, server) = github_api_server(vec![
        (
            200,
            r#"[{"name":"v2","commit":{"sha":"7184910d9eb2b1c5e48f7073824a90609bb9b6d6"}}]"#,
        ),
        (
            200,
            r#"{"ref":"refs/tags/v2","object":{"type":"tag","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#,
        ),
    ]);

    let session = crate::support::tests::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: GitHub,
                url: base_url,
            }],
            ..crate::default()
        },
        false,
    );
    let output = session.resolve_document(DocumentInput::new(
        "file:///work/.github/workflows/publish.yml".to_owned(),
        "yaml".to_owned(),
        "steps:\n  - uses: azure/login@8216e11d8cd9b42fe925c852af8e76311ff067ac # v2\n".to_owned(),
        None,
    ));
    let paths = server.join().unwrap();

    assert_eq!(
        paths,
        [
            "/repos/azure/login/tags",
            "/repos/azure/login/git/ref/tags/v2"
        ]
    );
    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("2"));
}

#[test]
fn missing_exact_github_ref_keeps_a_sha_annotation_unproven_without_an_error() {
    let (base_url, server) = github_api_server(vec![
        (
            200,
            r#"[{"name":"v2","commit":{"sha":"7184910d9eb2b1c5e48f7073824a90609bb9b6d6"}}]"#,
        ),
        (404, r#"{"message":"Not Found"}"#),
    ]);
    let session = crate::support::tests::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: GitHub,
                url: base_url,
            }],
            ..crate::default()
        },
        false,
    );

    let output = session.resolve_document(DocumentInput::new(
        "file:///work/.github/workflows/publish.yml".to_owned(),
        "yaml".to_owned(),
        "steps:\n  - uses: azure/login@8216e11d8cd9b42fe925c852af8e76311ff067ac # missing-v2\n"
            .to_owned(),
        None,
    ));
    let paths = server.join().unwrap();

    assert_eq!(
        paths,
        [
            "/repos/azure/login/tags",
            "/repos/azure/login/git/ref/tags/missing-v2"
        ]
    );
    assert!(output.edits.is_empty());
    assert!(output.suggestions.iter().all(|item| item.status != "error"));
}

#[test]
fn sha_pinned_action_updates_sha_and_annotation_atomically() {
    let current = "3d3c42e5aac5ba805825da76410c181273ba90b1";
    let target = "7777777777777777777777777777777777777777";
    let source = format!("steps:\n  - uses: actions/checkout@{current} # v7.0.1\n");
    let body = format!(
        r#"[{{"name":"v7.1.0","commit":{{"sha":"{target}"}}}},{{"name":"v7.0.1","commit":{{"sha":"{current}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
    );
    let output = resolve_action_source(&source, &body);

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, format!("{target} # v7.1.0"));
    let choices = action_choices(&source, "7.1.0", &body);
    assert!(choices.iter().any(|choice| choice.version == "6.0.0"
        && choice.replacement.as_deref()
            == Some("6666666666666666666666666666666666666666 # v6.0.0")));
}

#[test]
fn abbreviated_sha_pin_is_proven_by_the_full_tag_commit() {
    let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@3d3c42e # v7.0.1\n",
        r#"[{"name":"v7.1.0","commit":{"sha":"7777777777777777777777777777777777777777"}},{"name":"v7.0.1","commit":{"sha":"3d3c42e5aac5ba805825da76410c181273ba90b1"}}]"#,
    );

    assert_eq!(
        output.edits[0].new_text,
        "7777777777777777777777777777777777777777 # v7.1.0"
    );
}

#[test]
fn mismatched_sha_and_annotation_are_not_treated_as_a_proven_current_ref() {
    let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@3d3c42e # v7.0.1\n",
        r#"[{"name":"v7.1.0","commit":{"sha":"7777777777777777777777777777777777777777"}},{"name":"v7.0.1","commit":{"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}]"#,
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
fn action_tags_update_only_within_their_path_qualified_tag_lineage() {
    let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@release/action-v2.3.0\n",
        r#"[{"name":"v99.0.0"},{"name":"release/action-v2.4.0"},{"name":"release/action-v2.3.0"}]"#,
    );

    assert_eq!(output.edits[0].new_text, "release/action-v2.4.0");
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
        Some("1.3.0".to_owned())
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
