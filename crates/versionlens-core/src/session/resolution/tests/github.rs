use super::{DocumentInput, Ecosystem, RegistryResponseInput, parse_document, standard_session};

#[test]
fn resolves_npm_github_dependencies_from_tags() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"core.js":"github:octokit/core.js#semver:^1"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "octokit/core.js".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"commit":"github:owner/commit#1234567"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text:
                r#"{"dependencies":{"commit":"git+https://github.com/owner/commit.git#1234567"}}"#
                    .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text:
                r#"{"dependencies":{"commit":"git+ssh://git@github.com/owner/commit.git#1234567"}}"#
                    .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text:
                r#"{"dependencies":{"commit":"git+ssh://git@github.com:owner/commit.git#1234567"}}"#
                    .to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/commit".to_owned(),
            ecosystem: Ecosystem::Npm,
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
            text: r#"{"dependencies":{"bare":"github:owner/bare"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "owner/bare".to_owned(),
            ecosystem: Ecosystem::Npm,
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
        text: r#"{"dependencies":{"core.js":"github:octokit/core.js#semver:^1"}}"#.to_owned(),
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
        text: r#"{"dependencies":{"bare":"github:owner/bare"}}"#.to_owned(),
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
        text: r#"{"dependencies":{"commit":"github:owner/commit#abcdef1"}}"#.to_owned(),
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
            text: r#"gem "rspec-rails", github: "rspec/rspec-rails", tag: "v6.0.1""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rspec/rspec-rails".to_owned(),
            ecosystem: Ecosystem::Ruby,
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
            text: r#"gem "devise", github: "heartcombo/devise""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "heartcombo/devise".to_owned(),
            ecosystem: Ecosystem::Ruby,
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
            text: r#"gem "rspec-core", github: "rspec/rspec-core", branch: "main""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rspec/rspec-core".to_owned(),
            ecosystem: Ecosystem::Ruby,
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
            text: r#"gem "rails", git: "git@github.com:rails/rails.git", tag: "v7.0.0""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rails/rails".to_owned(),
            ecosystem: Ecosystem::Ruby,
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
            text: r#"gem "rails", git: "https://github.com/rails/rails.git""#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "rails/rails".to_owned(),
            ecosystem: Ecosystem::Ruby,
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
        text: r#"gem "rspec-core", github: "rspec/rspec-core", ref: "abcdef1""#.to_owned(),
        workspace_root: None,
    });

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://api.github.com/repos/rspec/rspec-core/commits"]
    );
}
