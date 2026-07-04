use super::{DocumentInput, parse_document};
use crate::document::test_support::extract_range;

#[test]
fn parses_package_json_github_dependencies() {
    let text = r#"{
  "dependencies": {
    "core.js": "github:octokit/core.js#semver:^2",
    "plain": "github:owner/plain#v1.0.0",
    "commit": "github:owner/commit#abcdef1",
    "shortcut": "owner/shortcut#v2.0.0",
    "url": "git+https://github.com/owner/url.git#semver:^3",
    "ssh": "git@github.com:owner/ssh.git#1234567",
    "git+ssh": "git+ssh://git@github.com/owner/git-ssh.git#7654321",
    "bare": "github:owner/bare"
  }
}"#;
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
    let text = r#"{
  "dependencies": {
    "git+ssh-colon": "git+ssh://git@github.com:owner/git-ssh-colon.git#89abcde"
  }
}"#;
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
    let text = r#"{
  "dependencies": {
    "git-url": "git+https://github.com/owner/url.git"
  }
}"#;
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
    let text = r#"{
  "dependencies": {
    "branch": "github:owner/branch#main"
  }
}"#;
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
