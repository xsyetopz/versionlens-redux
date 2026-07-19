use crate::maven_xml::MavenNamedRepository;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;
type GradleDependencies = Vec<Dependency>;

use crate::positions::offset_range;

pub type GradleMavenRepositories = Vec<MavenNamedRepository>;
type ParsedGradleDependency = Option<Dependency>;
type GradleBlockStack<'a> = Vec<&'a str>;

pub(crate) fn parse_gradle_build(text: &str) -> GradleDependencies {
    parse_gradle_script(text, true)
}

pub(crate) fn parse_gradle_settings(text: &str) -> GradleDependencies {
    parse_gradle_script(text, false)
}

pub fn parse_gradle_maven_repositories(text: &str) -> GradleMavenRepositories {
    parse_gradle_maven_repositories_with_scope(text, GradleAll)
}

pub fn parse_gradle_dependency_maven_repositories(text: &str) -> GradleMavenRepositories {
    parse_gradle_maven_repositories_with_scope(text, GradleDependency)
}

pub fn parse_gradle_plugin_maven_repositories(text: &str) -> GradleMavenRepositories {
    parse_gradle_maven_repositories_with_scope(text, GradlePluginManagement)
}

use GradleRepositoryScope::{
    All as GradleAll, Dependency as GradleDependency, PluginManagement as GradlePluginManagement,
};

#[derive(Clone, Copy)]
enum GradleRepositoryScope {
    All,
    Dependency,
    PluginManagement,
}

fn parse_gradle_maven_repositories_with_scope(
    text: &str,
    scope: GradleRepositoryScope,
) -> GradleMavenRepositories {
    let mut repositories = vec![];
    let mut block_stack = vec![];
    let mut lines = text.lines();

    while let Some(line) = lines.next() {
        pop_closed_blocks(line, &mut block_stack);

        let trimmed = line.trim_start();
        let in_repositories = block_stack.contains(&"repositories")
            && !block_stack.contains(&"publishing")
            && match scope {
                GradleAll => true,
                GradleDependency => !block_stack.contains(&"pluginManagement"),
                GradlePluginManagement => block_stack.contains(&"pluginManagement"),
            };

        if in_repositories && trimmed.starts_with("google()") {
            repositories.push(MavenNamedRepository {
                id: "google".to_owned(),
                url: "https://dl.google.com/dl/android/maven2/".to_owned(),
            });
        }

        if in_repositories && trimmed.starts_with("mavenCentral()") {
            repositories.push(MavenNamedRepository {
                id: "mavenCentral".to_owned(),
                url: "https://repo.maven.apache.org/maven2".to_owned(),
            });
        }

        if in_repositories && trimmed.starts_with("gradlePluginPortal()") {
            repositories.push(MavenNamedRepository {
                id: "gradlePluginPortal".to_owned(),
                url: "https://plugins.gradle.org/m2/".to_owned(),
            });
        }

        if in_repositories && starts_gradle_block(trimmed, "maven") {
            let mut block = line.to_owned();
            let mut balance = brace_delta(line);
            while balance > 0 {
                let Some(next_line) = lines.next() else {
                    break;
                };
                block.push('\n');
                block.push_str(next_line);
                balance += brace_delta(next_line);
            }

            if let Some(url) = gradle_repository_url(&block) {
                repositories.push(MavenNamedRepository {
                    id: "maven".to_owned(),
                    url: url.to_owned(),
                });
            }
            continue;
        }

        push_opened_blocks(trimmed, &mut block_stack);
    }

    repositories
}

fn parse_gradle_script(text: &str, include_dependencies: bool) -> GradleDependencies {
    let mut dependencies = vec![];
    let mut line_offset = 0;

    for line in text.lines() {
        if let Some(plugin) = parse_plugin_line(text, line, line_offset) {
            dependencies.push(plugin);
        }
        if include_dependencies {
            if let Some(dependency) = parse_string_dependency_line(text, line, line_offset) {
                dependencies.push(dependency);
            } else if let Some(dependency) = parse_map_dependency_line(text, line, line_offset) {
                dependencies.push(dependency);
            } else if let Some(dependency) = parse_local_dependency_line(text, line, line_offset) {
                dependencies.push(dependency);
            }
        }
        line_offset += line.len() + 1;
    }

    dependencies
}

fn parse_plugin_line(text: &str, line: &str, line_offset: usize) -> ParsedGradleDependency {
    let trimmed = line.trim_start();
    if !trimmed.starts_with("id") || !trimmed.contains("version") {
        return None;
    }

    let strings = quoted_strings(line);
    if strings.len() < 2 {
        return None;
    }

    let id = strings[0].value;
    let version = strings[1].value;
    let between = line.get(strings[0].end..strings[1].start)?;
    if !between.contains("version") {
        return None;
    }

    Some(Dependency {
        name: format!("{id}:{id}.gradle.plugin"),
        requirement: version.to_owned(),
        ecosystem: Maven,
        group: "plugins".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(
            text,
            line_offset + strings[0].start,
            line_offset + strings[1].end,
        ),
        requirement_range: offset_range(
            text,
            line_offset + strings[1].content_start,
            line_offset + strings[1].content_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn parse_string_dependency_line(
    text: &str,
    line: &str,
    line_offset: usize,
) -> ParsedGradleDependency {
    let trimmed = line.trim_start();
    if trimmed.starts_with("id") {
        return None;
    }

    let strings = quoted_strings(line);
    let gav = strings
        .iter()
        .find(|string| string.value.matches(':').count() >= 2)?;
    let mut parts = gav.value.split(':');
    let group_id = parts.next()?;
    let artifact_id = parts.next()?;
    let version = parts.next()?;
    if group_id.is_empty() || artifact_id.is_empty() || version.is_empty() {
        return None;
    }

    let configuration = dependency_configuration(trimmed)?;
    let version_content_start = gav.content_start + group_id.len() + 1 + artifact_id.len() + 1;
    let version_content_end = version_content_start + version.len();

    Some(Dependency {
        name: format!("{group_id}:{artifact_id}"),
        requirement: version.to_owned(),
        ecosystem: Maven,
        group: configuration.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, line_offset + gav.start, line_offset + gav.end),
        requirement_range: offset_range(
            text,
            line_offset + version_content_start,
            line_offset + version_content_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn parse_map_dependency_line(text: &str, line: &str, line_offset: usize) -> ParsedGradleDependency {
    let trimmed = line.trim_start();
    let configuration = dependency_configuration(trimmed)?;
    let group_id = named_string(line, "group")?;
    let artifact_id = named_string(line, "name")?;
    let version = named_string(line, "version")?;

    Some(Dependency {
        name: format!("{}:{}", group_id.value, artifact_id.value),
        requirement: version.value.to_owned(),
        ecosystem: Maven,
        group: configuration.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(
            text,
            line_offset + group_id.start,
            line_offset + version.end,
        ),
        requirement_range: offset_range(
            text,
            line_offset + version.content_start,
            line_offset + version.content_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn parse_local_dependency_line(
    text: &str,
    line: &str,
    line_offset: usize,
) -> ParsedGradleDependency {
    let trimmed = line.trim_start();
    let configuration = dependency_configuration(trimmed)?;
    let source = if trimmed.contains("project(") {
        "project"
    } else if trimmed.contains("files(") || trimmed.contains("fileTree(") {
        "file"
    } else {
        return None;
    };
    let local = quoted_strings(line).into_iter().next()?;

    Some(Dependency {
        name: local.value.to_owned(),
        requirement: local.value.to_owned(),
        ecosystem: Maven,
        group: configuration.to_owned(),
        hosted_url: Some("local".to_owned()),
        hosted_name: Some(source.to_owned()),
        range: offset_range(text, line_offset + local.start, line_offset + local.end),
        requirement_range: offset_range(
            text,
            line_offset + local.content_start,
            line_offset + local.content_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn pop_closed_blocks(line: &str, block_stack: &mut GradleBlockStack<'static>) {
    for character in line.trim_start().chars() {
        if character == '}' {
            block_stack.pop();
        } else {
            break;
        }
    }
}

fn push_opened_blocks(trimmed: &str, block_stack: &mut GradleBlockStack<'static>) {
    for block in ["pluginManagement", "publishing", "repositories"] {
        if starts_gradle_block(trimmed, block) {
            block_stack.push(block);
            return;
        }
    }
}

fn starts_gradle_block(trimmed: &str, name: &str) -> bool {
    let Some(rest) = trimmed.strip_prefix(name) else {
        return false;
    };
    rest.trim_start().starts_with('{')
}

fn brace_delta(line: &str) -> i32 {
    line.chars().fold(0, |balance, character| match character {
        '{' => balance + 1,
        '}' => balance - 1,
        _ => balance,
    })
}

fn gradle_repository_url(block: &str) -> Option<&str> {
    let url_start = block.find("url")?;
    quoted_strings(block.get(url_start..)?)
        .into_iter()
        .next()
        .map(|string| string.value)
}

fn dependency_configuration(trimmed: &str) -> Option<&str> {
    let end = trimmed
        .find(|character: char| character.is_whitespace() || character == '(')
        .unwrap_or(trimmed.len());
    let configuration = trimmed.get(..end)?;
    if configuration.is_empty()
        || matches!(
            configuration,
            "dependencies" | "plugins" | "repositories" | "project" | "files" | "fileTree"
        )
    {
        return None;
    }
    Some(configuration)
}

fn named_string<'a>(line: &'a str, name: &str) -> Option<QuotedString<'a>> {
    let colon_needle = format!("{name}:");
    let equals_needle = format!("{name} =");
    let (match_start, needle_len) = line
        .find(&colon_needle)
        .map(|start| (start, colon_needle.len()))
        .or_else(|| {
            line.find(&equals_needle)
                .map(|start| (start, equals_needle.len()))
        })?;
    let start = match_start + needle_len;
    quoted_strings(line.get(start..)?)
        .into_iter()
        .next()
        .map(|string| QuotedString {
            value: string.value,
            start: start + string.start,
            end: start + string.end,
            content_start: start + string.content_start,
            content_end: start + string.content_end,
        })
}

#[derive(Clone, Copy)]
struct QuotedString<'a> {
    value: &'a str,
    start: usize,
    end: usize,
    content_start: usize,
    content_end: usize,
}

fn quoted_strings(line: &str) -> Vec<QuotedString<'_>> {
    let mut strings = vec![];
    let mut search_start = 0;

    while let Some(relative_start) = line
        .get(search_start..)
        .and_then(|tail| tail.find(['\'', '"']))
    {
        let start = search_start + relative_start;
        let quote = line.as_bytes()[start] as char;
        let content_start = start + 1;
        let Some(relative_end) = line.get(content_start..).and_then(|tail| tail.find(quote)) else {
            break;
        };
        let content_end = content_start + relative_end;
        let end = content_end + 1;
        let Some(value) = line.get(content_start..content_end) else {
            break;
        };
        strings.push(QuotedString {
            value,
            start,
            end,
            content_start,
            content_end,
        });
        search_start = end;
    }

    strings
}
