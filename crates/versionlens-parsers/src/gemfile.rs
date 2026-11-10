use self::line::gem_name_range;
use versionlens_model::Ecosystem::Ruby;
mod github;
mod line;
mod sources;
mod standard;
mod syntax;

use crate::positions::line_range;
use github::{gem_github_default_dependency, gem_github_ref_dependency, gem_github_tag_dependency};
use line::{GemLineContext, GemNameSpan};
use sources::source_block_url_from_line;
use standard::standard_gem_dependency;
use syntax::{quoted_strings, strip_comment};
use versionlens_model::Dependency;

pub use sources::parse_gemfile_source_urls;

pub(crate) fn parse_gemfile(text: &str) -> Vec<Dependency> {
    let mut parser: GemfileParser<'_> = crate::default();

    for (line_index, line) in text.lines().enumerate() {
        parser.collect_line(line_index, line);
    }

    parser.dependencies
}

pub(crate) fn parse_gemspec(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_gemspec_line(line_index, line))
        .collect()
}

#[derive(Default)]
struct GemfileParser<'a> {
    block_stack: Vec<GemfileBlock<'a>>,
    dependencies: Vec<Dependency>,
}

enum GemfileBlock<'a> {
    Group(&'a str),
    Source(String),
    Git { url: String, group: String },
    Path { path: String, group: String },
}

impl<'a> GemfileParser<'a> {
    fn collect_line(&mut self, line_index: usize, line: &'a str) {
        use GemfileBlock::{Git, Group, Path, Source};
        let trimmed = line.trim_start();
        if trimmed.trim_end() == "end" {
            self.block_stack.pop();
            return;
        }
        if let Some(url) = source_block_url_from_line(trimmed) {
            self.block_stack.push(Source(url));
            return;
        }
        if let Some(path) = block_quoted_value(trimmed, "path") {
            self.block_stack.push(Path {
                path: path.to_owned(),
                group: format!("path {path}"),
            });
            return;
        }
        if let Some(url) = block_quoted_value(trimmed, "git") {
            self.block_stack.push(Git {
                url: url.to_owned(),
                group: format!("git {url}"),
            });
            return;
        }
        if trimmed.starts_with("group ") && trimmed.ends_with(" do") {
            self.block_stack
                .push(Group(trimmed.trim_end_matches(" do")));
            return;
        }

        if let Some(dependency) = parse_gem_line(
            line_index,
            line,
            self.current_group(),
            self.current_source(),
            self.current_block_requirement(),
        ) {
            self.dependencies.push(dependency);
        }
    }

    fn current_group(&self) -> &str {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack
            .iter()
            .rev()
            .find_map(|block| match block {
                Group(group) => Some(*group),
                Source(_) => None,
                Git { group, .. } | Path { group, .. } => Some(group.as_str()),
            })
            .unwrap_or("dependencies")
    }

    fn current_source(&self) -> Option<&str> {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack.iter().rev().find_map(|block| match block {
            Source(url) => Some(url.as_str()),
            Group(_) | Git { .. } | Path { .. } => None,
        })
    }

    fn current_block_requirement(&self) -> Option<&str> {
        use GemfileBlock::{Git, Group, Path, Source};
        self.block_stack.iter().rev().find_map(|block| match block {
            Git { url, .. } => Some(url.as_str()),
            Path { path, .. } => Some(path.as_str()),
            Group(_) | Source(_) => None,
        })
    }
}

fn parse_gem_line(
    line_index: usize,
    line: &str,
    group: &str,
    source_url: Option<&str>,
    block_requirement: Option<&str>,
) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("gem ") {
        return None;
    }

    let content = strip_comment(trimmed).trim_end();
    let inline_group = inline_group_name(content);
    let group_name = inline_group.as_deref().unwrap_or(group);
    let gem_context = GemLineContext {
        line_index,
        line,
        offset: line.len() - trimmed.len(),
        content,
        group: group_name,
    };
    let mut strings = quoted_strings(gem_context.content);
    let (name, _, end) = strings.next()?;
    let name = GemNameSpan { name, end };

    if let Some(requirement) = block_requirement {
        return Some(inherited_block_dependency(&gem_context, &name, requirement));
    }

    gem_github_tag_dependency(&gem_context, &name)
        .or_else(|| gem_github_ref_dependency(&gem_context, &name))
        .or_else(|| gem_github_default_dependency(&gem_context, &name))
        .or_else(|| {
            Some(standard_gem_dependency(
                &gem_context,
                &name,
                strings.next(),
                source_url,
            ))
        })
}

fn inline_group_name(content: &str) -> Option<String> {
    inline_group_value(content, "groups")
        .map(|value| format!("groups {value}"))
        .or_else(|| inline_group_value(content, "group").map(|value| format!("group {value}")))
}

fn inline_group_value(content: &str, attr: &str) -> Option<String> {
    let marker = format!("{attr}:");
    let attr_start = content.find(&marker)?;
    let value = content[attr_start + marker.len()..].trim_start();
    if let Some(array_tail) = value.strip_prefix('[') {
        let array_end = array_tail.find(']')?;
        let array = &array_tail[..array_end];
        let quoted = quoted_strings(array)
            .map(|(value, _, _)| value.to_owned())
            .collect::<Vec<_>>();
        if !quoted.is_empty() {
            return Some(quoted.join(", "));
        }
        let symbols = array
            .split(',')
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        return (!symbols.is_empty()).then(|| symbols.join(", "));
    }

    let value = value
        .split([',', ' '])
        .next()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())?;
    Some(value.to_owned())
}

fn block_quoted_value<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if !(line.starts_with(keyword) && line.ends_with(" do")) {
        return None;
    }
    let (value, _, _) = quoted_strings(line).next()?;
    Some(value)
}

fn inherited_block_dependency(
    context: &GemLineContext<'_>,
    name: &GemNameSpan<'_>,
    requirement: &str,
) -> Dependency {
    let quote = context
        .content
        .as_bytes()
        .get(name.end)
        .copied()
        .unwrap_or(b'"') as char;
    let insert_at = name.end + quote.len_utf8();
    Dependency {
        name: name.name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ruby,
        group: context.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: gem_name_range(context),
        requirement_range: line_range(
            context.line_index,
            context.line,
            context.offset + insert_at,
            context.offset + insert_at,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

fn parse_gemspec_line(line_index: usize, line: &str) -> Option<Dependency> {
    const METHODS: [&str; 3] = [
        "add_runtime_dependency",
        "add_development_dependency",
        "add_dependency",
    ];

    let trimmed = line.trim_start();
    let content = strip_comment(trimmed).trim_end();
    let (method, _method_start, method_end) = METHODS.iter().find_map(|method| {
        let start = content.find(method)?;
        let before = content[..start].trim_end();
        let method_start = start + content[start..].find(method)?;
        let valid_receiver = before.is_empty() || before.ends_with('.');
        valid_receiver.then_some((*method, method_start, method_start + method.len()))
    })?;

    let args = &content[method_end..];
    let args_offset = method_end;
    let mut strings = quoted_strings(args);
    let (name, name_start, name_end) = strings.next()?;
    let version = strings.next();
    let (requirement, requirement_start, requirement_end, requirement_prefix, requirement_suffix) =
        if let Some((requirement, start, end)) = version {
            (
                requirement.to_owned(),
                args_offset + start,
                args_offset + end,
                "".to_owned(),
                "".to_owned(),
            )
        } else {
            let quote = args.as_bytes().get(name_end).copied().unwrap_or(b'\"') as char;
            let insert_at = args_offset + name_end + quote.len_utf8();
            (
                "*".to_owned(),
                insert_at,
                insert_at,
                format!(", {quote}"),
                quote.to_string(),
            )
        };
    let offset = line.len() - trimmed.len();

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: Ruby,
        group: method.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(
            line_index,
            line,
            offset + args_offset + name_start,
            offset + args_offset + name_end,
        ),
        requirement_range: line_range(
            line_index,
            line,
            offset + requirement_start,
            offset + requirement_end,
        ),
        requirement_prefix,
        requirement_suffix,
        canonical_reference: None,
    })
}

#[cfg(test)]
mod tests {
use crate::document::test_support::{extract_range, parse_fixture};
use crate::{DocumentInput, parse_document};
use versionlens_model::Ecosystem::Ruby;

#[test]
fn parses_gemfile_dependencies() {
    let text = package_file_fixture("parses-gemfile-dependencies.txt");
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 11);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Ruby,
            "dependencies",
            "rails",
            "8.1.3",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "8.1.3"
    );
    assert_eq!(dependencies[1].name, "puma");
    assert_eq!(dependencies[1].requirement, ">= 8.0.2");
    assert_eq!(dependencies[2].name, "rails/rails");
    assert_eq!(dependencies[2].requirement, "");
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("branch"));
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
    assert_eq!(dependencies[7].hosted_url.as_deref(), Some("branch"));
    assert_eq!(
        extract_range(text, dependencies[7].requirement_range),
        "main"
    );
    assert_eq!(dependencies[7].requirement_prefix, "");
    assert_eq!(dependencies[7].requirement_suffix, "");
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
    let dependencies = parse_gemfile(text);

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
    let dependencies = parse_gemfile(text);

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
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "nokogiri");
    assert_eq!(dependencies[0].requirement, "*");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_range.start.character, 14);
    assert_eq!(dependencies[0].requirement_prefix, ", '");
    assert_eq!(dependencies[0].requirement_suffix, "'");
}

#[test]
fn parses_gemfile_github_dependencies_without_ref_as_branch() {
    let text =
        package_file_fixture("parses-gemfile-github-dependencies-without-ref-from-commitsGemfile");
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "heartcombo/devise");
    assert_eq!(dependencies[0].requirement, "");
    assert_github_branch_dependency(&dependencies, text);
}

#[test]
fn parses_gemfile_git_github_tag_and_ref_dependencies() {
    let text = package_file_fixture("parses-gemfile-git-github-tag-and-ref-dependencies.txt");
    let dependencies = parse_gemfile(text);

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
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("branch"));
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "main"
    );
    assert_eq!(dependencies[1].requirement_prefix, "");
    assert_eq!(dependencies[1].requirement_suffix, "");
}

#[test]
fn parses_gemfile_git_github_dependencies_without_ref() {
    let text = package_file_fixture("parses-gemfile-git-github-dependencies-without-ref.txt");
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "rails/rails");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("branch"));
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("rails"));
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_prefix, r#", ref: ""#);
    assert_eq!(dependencies[0].requirement_suffix, r#"""#);
}

#[test]
fn parses_smoke_gemfile_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-gemfile-smoke-shapes.txt");
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Ruby);
    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(dependencies[0].requirement, "8.1.3");
    assert_eq!(dependencies[5].name, "byebug");
    assert_eq!(dependencies[5].requirement, "13.0.0");
    assert_eq!(dependencies[9].name, "rails/rails");
    assert_eq!(dependencies[9].requirement, "");
    assert_eq!(dependencies[9].hosted_url.as_deref(), Some("branch"));
    assert_eq!(dependencies[10].group, "group :production");
    assert_eq!(dependencies[10].name, "pg");
    assert_eq!(dependencies[11].name, "rails_12factor");
}

#[test]
fn parses_smoke_gemfile_github_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-gemfile-github-smoke-shapes.txt");
    let dependencies = parse_gemfile(text);

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
    let dependencies = parse_gemfile(text);

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
    let dependencies = parse_gemfile(text);

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "rspec");
    assert_eq!(dependencies[0].group, "group :test");
    assert_eq!(dependencies[1].name, "rails");
    assert_eq!(dependencies[1].group, "dependencies");
}

#[test]
fn parses_gemspec_dependencies() {
    let text = package_file_fixture("parses-gemspec-dependenciesGemfile");
    let dependencies = parse_fixture(text, "file:///work/example.gemspec", "ruby");

    assert_eq!(dependencies.len(), 4);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Ruby,
            "add_dependency",
            "rack",
            "~> 2.2",
        ),
    );
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
    let dependencies = parse_gemfile(text);

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
    let dependencies = parse_gemfile(text);

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

fn parse_gemfile(text: impl AsRef<str>) -> Vec<versionlens_model::Dependency> {
    parse_document(&DocumentInput::new(
        "file:///work/Gemfile".to_owned(),
        "ruby".to_owned(),
        text.as_ref().to_owned(),
        None,
    ))
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/gemfile/tests", name)
}

fn assert_github_branch_dependency(dependencies: &[versionlens_model::Dependency], text: &str) {
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("branch"));
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[0].requirement_prefix, r#", ref: ""#);
    assert_eq!(dependencies[0].requirement_suffix, r#"""#);
}
}
