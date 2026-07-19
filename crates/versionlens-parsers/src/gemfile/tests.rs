use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::Ruby;

#[test]
fn parses_gemfile_dependencies() {
    let text = package_file_fixture("parses-gemfile-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 11);
    assert_eq!(dependencies[0].ecosystem, Ruby);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(dependencies[0].requirement, "8.1.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "8.1.3"
    );
    assert_eq!(dependencies[1].name, "puma");
    assert_eq!(dependencies[1].requirement, ">= 8.0.2");
    assert_eq!(dependencies[2].name, "rails/rails");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(
        dependencies[2].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rails/rails/commits")
    );
    assert_eq!(dependencies[2].hosted_name.as_deref(), Some("rails"));
    assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
    assert_eq!(dependencies[2].requirement_prefix, r#", ref: ""#);
    assert_eq!(dependencies[2].requirement_suffix, r#"""#);
    assert_eq!(dependencies[3].name, "sqlite3");
    assert_eq!(dependencies[3].requirement, "*");
    assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
    assert_eq!(dependencies[3].requirement_prefix, r#", ""#);
    assert_eq!(dependencies[3].requirement_suffix, r#"""#);
    assert_eq!(dependencies[4].name, "local");
    assert_eq!(dependencies[4].requirement, "vendor/local");
    assert_eq!(
        extract_range(text, dependencies[4].requirement_range),
        "vendor/local"
    );
    assert_eq!(dependencies[5].name, "rspec/rspec-rails");
    assert_eq!(dependencies[5].requirement, "v6.0.1");
    assert_eq!(
        dependencies[5].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rspec/rspec-rails/tags")
    );
    assert_eq!(dependencies[5].hosted_name.as_deref(), Some("rspec-rails"));
    assert_eq!(
        extract_range(text, dependencies[5].requirement_range),
        r#"tag: "v6.0.1""#
    );
    assert_eq!(dependencies[6].name, "rspec/rspec-core");
    assert_eq!(dependencies[6].requirement, "abcdef1");
    assert_eq!(
        dependencies[6].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rspec/rspec-core/commits")
    );
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "abcdef1"
    );
    assert_eq!(dependencies[7].name, "rspec/rspec-mocks");
    assert_eq!(dependencies[7].requirement, "main");
    assert_eq!(
        dependencies[7].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rspec/rspec-mocks/commits")
    );
    assert_eq!(
        extract_range(text, dependencies[7].requirement_range),
        r#"branch: "main""#
    );
    assert_eq!(dependencies[7].requirement_prefix, r#"ref: ""#);
    assert_eq!(dependencies[7].requirement_suffix, r#"""#);
    assert_eq!(dependencies[8].name, "fragment");
    assert_eq!(
        dependencies[8].requirement,
        "https://example.test/repo.git#main"
    );
    assert_eq!(dependencies[9].name, "quoted-comment");
    assert_eq!(dependencies[9].requirement, "*");
    assert_eq!(dependencies[9].requirement_prefix, r#", ""#);
    assert_eq!(dependencies[9].requirement_suffix, r#"""#);
    assert_eq!(dependencies[10].group, "group :production");
    assert_eq!(dependencies[10].name, "pg");
    assert_eq!(dependencies[10].requirement, "1.6.2");
}

#[test]
fn parses_gemfile_source_block_dependencies() {
    let text = package_file_fixture("parses-gemfile-source-block-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://private.gems.example.test")
    );
    assert_eq!(dependencies[1].group, "group :development");
    assert_eq!(dependencies[1].name, "dev_private");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("https://private.gems.example.test")
    );
    assert_eq!(dependencies[2].name, "owner/remote");
    assert_eq!(
        dependencies[2].hosted_url.as_deref(),
        Some("https://api.github.com/repos/owner/remote/tags")
    );
}

#[test]
fn parses_gemfile_dependency_source_option() {
    let text = package_file_fixture("parses-gemfile-dependency-source-option.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://block.gems.example.test")
    );
    assert_eq!(dependencies[1].name, "explicit_private");
    assert_eq!(dependencies[1].requirement, "2.0.0");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("https://explicit.gems.example.test")
    );
    assert_eq!(dependencies[2].name, "standalone_private");
    assert_eq!(dependencies[2].requirement, "*");
    assert_eq!(
        dependencies[2].hosted_url.as_deref(),
        Some("https://standalone.gems.example.test")
    );
}

#[test]
fn parses_gemfile_missing_single_quote_version_insert() {
    let text = package_file_fixture("parses-gemfile-missing-single-quote-version-insert.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "nokogiri");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_range.start.character, 14);
    assert_eq!(dependencies[0].requirement_prefix, ", '");
    assert_eq!(dependencies[0].requirement_suffix, "'");
}

#[test]
fn parses_gemfile_github_dependencies_without_ref_from_commits() {
    let text =
        package_file_fixture("parses-gemfile-github-dependencies-without-ref-from-commitsGemfile");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "heartcombo/devise");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/heartcombo/devise/commits")
    );
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_prefix, r#", ref: ""#);
    assert_eq!(dependencies[0].requirement_suffix, r#"""#);
}

#[test]
fn parses_gemfile_git_github_tag_and_ref_dependencies() {
    let text = package_file_fixture("parses-gemfile-git-github-tag-and-ref-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "rails/rails");
    assert_eq!(dependencies[0].requirement, "v8.0.0");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rails/rails/tags")
    );
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("rails"));
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        r#"tag: "v8.0.0""#
    );
    assert_eq!(dependencies[0].requirement_prefix, r#"tag: ""#);
    assert_eq!(dependencies[0].requirement_suffix, r#"""#);
    assert_eq!(dependencies[1].name, "rspec/rspec-core");
    assert_eq!(dependencies[1].requirement, "main");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rspec/rspec-core/commits")
    );
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        r#"branch: "main""#
    );
    assert_eq!(dependencies[1].requirement_prefix, r#"ref: ""#);
    assert_eq!(dependencies[1].requirement_suffix, r#"""#);
}

#[test]
fn parses_gemfile_git_github_dependencies_without_ref() {
    let text = package_file_fixture("parses-gemfile-git-github-dependencies-without-ref.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rails/rails");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rails/rails/commits")
    );
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("rails"));
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_prefix, r#", ref: ""#);
    assert_eq!(dependencies[0].requirement_suffix, r#"""#);
}

#[test]
fn parses_smoke_gemfile_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-gemfile-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Ruby);
    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(dependencies[0].requirement, "8.1.3");
    assert_eq!(dependencies[5].name, "byebug");
    assert_eq!(dependencies[5].requirement, "13.0.0");
    assert_eq!(dependencies[9].name, "rails/rails");
    assert_eq!(dependencies[9].requirement, "");
    assert_eq!(
        dependencies[9].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rails/rails/commits")
    );
    assert_eq!(dependencies[10].group, "group :production");
    assert_eq!(dependencies[10].name, "pg");
    assert_eq!(dependencies[11].name, "rails_12factor");
}

#[test]
fn parses_smoke_gemfile_github_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-gemfile-github-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].name, "rspec/rspec-rails");
    assert_eq!(dependencies[0].requirement, "v8.0.4");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("rspec-rails"));
    assert_eq!(dependencies[1].name, "rails/rails");
    assert_eq!(dependencies[1].requirement, "9a475c8");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("https://api.github.com/repos/rails/rails/commits")
    );
    assert_eq!(dependencies[3].name, "thoughtbot/factory_bot");
    assert_eq!(dependencies[3].hosted_name.as_deref(), Some("factory_bot"));
}
#[test]
fn gemfile_dependency_range_starts_at_gem_keyword_like_upstream() {
    let text =
        package_file_fixture("gemfile-dependency-range-starts-at-gem-keyword-like-upstream.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(dependencies[0].range.start.character, 2);
    assert_eq!(dependencies[0].range.end.character, 2);
    assert_eq!(extract_range(text, dependencies[0].range), "");
}
#[test]
fn gemfile_group_end_accepts_trailing_whitespace_like_upstream() {
    let text =
        package_file_fixture("gemfile-group-end-accepts-trailing-whitespace-like-upstreamGemfile");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "rspec");
    assert_eq!(dependencies[0].group, "group :test");
    assert_eq!(dependencies[1].name, "rails");
    assert_eq!(dependencies[1].group, "dependencies");
}

#[test]
fn parses_gemspec_dependencies() {
    let text = package_file_fixture("parses-gemspec-dependenciesGemfile");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/example.gemspec".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Ruby);
    assert_eq!(dependencies[0].group, "add_dependency");
    assert_eq!(dependencies[0].name, "rack");
    assert_eq!(dependencies[0].requirement, "~> 2.2");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~> 2.2"
    );
    assert_eq!(dependencies[1].group, "add_runtime_dependency");
    assert_eq!(dependencies[1].name, "thor");
    assert_eq!(dependencies[1].requirement, ">= 1.0");
    assert_eq!(dependencies[2].group, "add_development_dependency");
    assert_eq!(dependencies[2].name, "rspec");
    assert_eq!(dependencies[2].requirement, "~> 3.13");
    assert_eq!(dependencies[3].group, "add_dependency");
    assert_eq!(dependencies[3].name, "json");
    assert_eq!(dependencies[3].requirement, "2.7.2");
}

#[test]
fn parses_gemfile_git_and_path_block_dependencies_as_non_registry_sources() {
    let text = package_file_fixture(
        "parses-gemfile-git-and-path-block-dependencies-as-non-registry-sources.txt",
    );
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "local_one");
    assert_eq!(dependencies[0].requirement, "vendor/local");
    assert_eq!(dependencies[0].group, "path vendor/local");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[1].name, "activesupport");
    assert_eq!(
        dependencies[1].requirement,
        "https://github.com/rails/rails.git"
    );
    assert_eq!(
        dependencies[1].group,
        "git https://github.com/rails/rails.git"
    );
    assert_eq!(extract_range(text, dependencies[1].requirement_range), "");
    assert_eq!(dependencies[2].name, "actionpack");
    assert_eq!(
        dependencies[2].requirement,
        "https://github.com/rails/rails.git"
    );
}

#[test]
fn parses_gemfile_inline_group_options_as_dependency_groups() {
    let text = package_file_fixture("parses-gemfile-inline-group-options-as-dependency-groups.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "rubocop");
    assert_eq!(dependencies[0].group, "group :development");
    assert_eq!(dependencies[0].requirement, "1.80.2");
    assert_eq!(dependencies[1].name, "rspec");
    assert_eq!(dependencies[1].group, "groups :development, :test");
    assert_eq!(dependencies[1].requirement, "3.13.0");
    assert_eq!(dependencies[2].name, "guard");
    assert_eq!(dependencies[2].group, "group development, test");
    assert_eq!(dependencies[2].requirement, "2.19.1");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/gemfile/tests")
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
