use super::{DocumentInput, parse_document};
use crate::document::test_support::extract_range;
use std::fs::read_to_string;
use std::path::PathBuf;

#[test]
fn parses_package_json_github_dependencies() {
    let text = package_file_fixture("parses-package-json-github-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].name, "octokit/core.js");
    assert_eq!(dependencies[0].requirement, "^2");
    assert_eq!(
        dependencies[0].requirement_prefix,
        "github:octokit/core.js#semver:"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "github:octokit/core.js#semver:^2"
    );
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/octokit/core.js/tags")
    );
    assert_eq!(dependencies[1].name, "owner/plain");
    assert_eq!(dependencies[1].requirement, "v1.0.0");
    assert_eq!(dependencies[1].requirement_prefix, "github:owner/plain#");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/plain/tags")
    );
    assert_eq!(dependencies[2].name, "owner/commit");
    assert_eq!(dependencies[2].requirement, "abcdef1");
    assert_eq!(dependencies[2].requirement_prefix, "github:owner/commit#");
    assert_eq!(
        dependencies[2].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/commit/commits")
    );
    assert_eq!(dependencies[3].name, "owner/shortcut");
    assert_eq!(dependencies[3].requirement, "v2.0.0");
    assert_eq!(dependencies[3].requirement_prefix, "owner/shortcut#");
    assert_eq!(
        dependencies[3].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/shortcut/tags")
    );
    assert_eq!(dependencies[4].name, "owner/url");
    assert_eq!(dependencies[4].requirement, "^3");
    assert_eq!(
        dependencies[4].requirement_prefix,
        "git+https://github.com/owner/url.git#semver:"
    );
    assert_eq!(
        dependencies[4].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/url/tags")
    );
    assert_eq!(dependencies[5].name, "owner/ssh");
    assert_eq!(dependencies[5].requirement, "1234567");
    assert_eq!(
        dependencies[5].requirement_prefix,
        "git@github.com:owner/ssh.git#"
    );
    assert_eq!(
        dependencies[5].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/ssh/commits")
    );
    assert_eq!(dependencies[6].name, "owner/git-ssh");
    assert_eq!(dependencies[6].requirement, "7654321");
    assert_eq!(
        dependencies[6].requirement_prefix,
        "git+ssh://git@github.com/owner/git-ssh.git#"
    );
    assert_eq!(
        dependencies[6].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/git-ssh/commits")
    );
    assert_eq!(dependencies[7].name, "owner/bare");
    assert_eq!(dependencies[7].requirement, "");
    assert_eq!(dependencies[7].requirement_prefix, "github:owner/bare#");
    assert_eq!(
        extract_range(text, dependencies[7].requirement_range),
        "github:owner/bare"
    );
    assert_eq!(
        dependencies[7].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/bare/commits")
    );
}

#[test]
fn parses_package_json_github_ssh_colon_dependencies() {
    let text = package_file_fixture("parses-package-json-github-ssh-colon-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "owner/git-ssh-colon");
    assert_eq!(dependencies[0].requirement, "89abcde");
    assert_eq!(
        dependencies[0].requirement_prefix,
        "git+ssh://git@github.com:owner/git-ssh-colon.git#"
    );
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/git-ssh-colon/commits")
    );
}

#[test]
fn parses_github_url_without_ref_as_plain_git_dependency() {
    let text = package_file_fixture("parses-github-url-without-ref-as-plain-git-dependency.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "git-url");
    assert_eq!(
        dependencies[0].requirement,
        "git+https://github.com/owner/url.git"
    );
    assert_eq!(dependencies[0].hosted_url, None);
}

#[test]
fn parses_package_json_github_branch_dependencies_as_commits() {
    let text =
        package_file_fixture("parses-package-json-github-branch-dependencies-as-commits.json");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "owner/branch");
    assert_eq!(dependencies[0].requirement, "main");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/branch/commits")
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_github")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
