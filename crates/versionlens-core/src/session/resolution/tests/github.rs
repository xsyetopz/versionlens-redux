use super::{DocumentInput, RegistryResponseInput, parse_document, standard_session};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::{Npm, Ruby};

#[test]
fn resolves_npm_github_dependencies_from_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("resolves-npm-github-dependencies-from-tags.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "octokit/core.js".to_owned(),
            ecosystem: Npm,
            body: r#"[{"name":"v2.5.0"},{"name":"v1.9.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(
        output.edits[0].new_text,
        "github:octokit/core.js#semver:2.5.0"
    );
}

#[test]
fn resolves_npm_github_commit_dependencies_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("resolves-npm-github-commit-dependencies-from-commits.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "github:owner/commit#abcdef1");
}

#[test]
fn resolves_npm_github_url_commit_dependencies_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-npm-github-url-commit-dependencies-from-commits.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(
        output.edits[0].new_text,
        "git+https://github.com/owner/commit.git#abcdef1"
    );
}

#[test]
fn resolves_npm_github_git_ssh_dependencies_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-npm-github-git-ssh-dependencies-from-commits.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(
        output.edits[0].new_text,
        "git+ssh://git@github.com/owner/commit.git#abcdef1"
    );
}

#[test]
fn resolves_npm_github_git_ssh_colon_dependencies_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-npm-github-git-ssh-colon-dependencies-from-commits.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(
        output.edits[0].new_text,
        "git+ssh://git@github.com:owner/commit.git#abcdef1"
    );
}

#[test]
fn resolves_npm_github_dependencies_without_refs_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "resolves-npm-github-dependencies-without-refs-from-commits.json",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/bare".to_owned(),
            ecosystem: Npm,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "github:owner/bare#abcdef1");
}

#[test]
fn routes_npm_github_tag_dependencies_to_tags() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("routes-npm-github-tag-dependencies-to-tags.json"),
        workspace_root: None,
    });

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/octokit/core.js/tags"]
    );
}

#[test]
fn routes_npm_github_dependencies_without_refs_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("routes-npm-github-dependencies-without-refs-to-commits.json"),
        workspace_root: None,
    });

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/owner/bare/commits"]
    );
}

#[test]
fn routes_npm_github_commit_dependencies_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("routes-npm-github-commit-dependencies-to-commits.json"),
        workspace_root: None,
    });

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/owner/commit/commits"]
    );
}

#[test]
fn resolves_ruby_github_tag_dependencies_from_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("resolves-ruby-github-tag-dependencies-from-tagsGemfile"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rspec/rspec-rails".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"name":"v6.1.0"},{"name":"v6.0.1"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"tag: "v6.1.0""#);
}

#[test]
fn resolves_ruby_github_dependencies_without_ref_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture(
                "resolves-ruby-github-dependencies-without-ref-from-commitsGemfile",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "heartcombo/devise".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#", ref: "abcdef1""#);
}

#[test]
fn resolves_ruby_github_ref_dependencies_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("resolves-ruby-github-ref-dependencies-from-commitsGemfile"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rspec/rspec-core".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"ref: "abcdef1""#);
}

#[test]
fn resolves_ruby_git_github_tag_dependencies_from_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture(
                "resolves-ruby-git-github-tag-dependencies-from-tagsGemfile",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rails/rails".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"name":"v8.0.0"},{"name":"v7.0.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#"tag: "v8.0.0""#);
}

#[test]
fn resolves_ruby_git_github_dependencies_without_ref_from_commits() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture(
                "resolves-ruby-git-github-dependencies-without-ref-from-commitsGemfile",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rails/rails".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, r#", ref: "abcdef1""#);
}

#[test]
fn routes_ruby_github_ref_dependencies_to_commits() {
    let session = standard_session();
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("routes-ruby-github-ref-dependencies-to-commitsGemfile"),
        workspace_root: None,
    });

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/rspec/rspec-core/commits"]
    );
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/github")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
