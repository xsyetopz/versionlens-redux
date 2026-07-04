use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_gemfile_dependencies() {
    let text = r#"
source "https://rubygems.org"
gem "rails", "8.1.3"
gem 'puma', '>= 8.0.2' # comment
gem "rails", git: "https://github.com/rails/rails.git"
gem "sqlite3"
gem "local", path: "vendor/local"
gem "rspec-rails", github: "rspec/rspec-rails", tag: "v6.0.1"
gem "rspec-core", github: "rspec/rspec-core", ref: "abcdef1"
gem "rspec-mocks", github: "rspec/rspec-mocks", branch: "main"
gem "fragment", git: "https://example.test/repo.git#main"
gem "quoted-comment" # "not-a-version"
group :production do
  gem "pg", "1.6.2"
end
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 11);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Ruby);
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
    let text = r#"
source "https://private.gems.example.test/" do
  gem "private_gem", "1.0.0"
  group :development do
    gem "dev_private", "2.0.0"
  end
  gem "remote", github: "owner/remote", tag: "v1.0.0"
end
"#;
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
    let text = r#"
source "https://block.gems.example.test/" do
  gem "block_private", "1.0.0"
  gem "explicit_private", "2.0.0", source: "https://explicit.gems.example.test/"
end
gem "standalone_private", source: "https://standalone.gems.example.test/"
"#;
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
    let text = "gem 'nokogiri'";
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
    let text = r#"gem "devise", github: "heartcombo/devise""#;
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
    let text = r#"gem "rails", git: "git@github.com:rails/rails.git", tag: "v8.0.0"
gem "core", git: "https://github.com/rspec/rspec-core.git", branch: "main"
"#;
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
    let text = r#"gem "rails", git: "https://github.com/rails/rails.git""#;
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
    let text = r#"# Gemfile for smoke testing
source 'https://rubygems.org'

gem 'rails', '8.1.3'
gem 'sqlite3', '~> 1.4'
gem 'puma', '>= 8.0.2'
gem 'bootsnap', '1.24.6'
gem 'sass-rails', '6.0.0'
gem 'byebug', '13.0.0' # test comment
gem 'ffaker', '2.25.0'
gem 'rspec-rails', '~> 8.0.4'
gem 'not_found_gem', '9.9.9'
gem 'rails', git: "https://github.com/rails/rails.git"

group :production do
  gem 'pg', '1.6.3'
  gem 'rails_12factor', '0.0.3'
end
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Ruby);
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
    let text = r#"source 'https://rubygems.org'

gem 'rspec-rails', github: 'rspec/rspec-rails', tag: 'v8.0.4'
gem 'rails', github: 'rails/rails', ref: '9a475c8'
gem 'devise', github: 'heartcombo/devise', ref: '372b295'
gem 'factory_bot', github: 'thoughtbot/factory_bot', ref: 'f923a81'
"#;
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
    let text = "  gem \"rails\", \"8.1.3\"\n";
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
    let text = "group :test do\n  gem \"rspec\", \"3.13.0\"\nend   \ngem \"rails\", \"8.1.3\"\n";
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
