use crate::maven_xml::MavenNamedRepository;
use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

pub(crate) fn parse_sbt_build(text: &str) -> Vec<Dependency> {
    let scala_binary_version = scala_binary_version(text);
    let mut dependencies = vec![];
    let mut line_offset = 0;
    let mut dependency_block_group: Option<&str> = None;
    let mut dependency_block_balance = 0;

    for line in text.lines() {
        let explicit_group = sbt_dependency_group(line);
        let group = explicit_group.or(dependency_block_group);
        if let Some(group) = group {
            dependencies.extend(parse_sbt_dependency_line(
                text,
                line,
                line_offset,
                group,
                scala_binary_version.as_deref(),
            ));
        }
        if let Some(group) = explicit_group {
            dependency_block_group = Some(group);
            dependency_block_balance = 0;
        }
        if dependency_block_group.is_some() {
            dependency_block_balance += paren_delta(line);
            if dependency_block_balance <= 0 {
                dependency_block_group = None;
                dependency_block_balance = 0;
            }
        }
        line_offset += line.len() + 1;
    }

    dependencies
}

pub fn parse_sbt_maven_repositories(text: &str) -> Vec<MavenNamedRepository> {
    text.lines()
        .filter_map(parse_sbt_maven_repository_line)
        .collect()
}

fn parse_sbt_maven_repository_line(line: &str) -> Option<MavenNamedRepository> {
    if !line.contains(" at ") {
        return None;
    }

    let strings = quoted_strings(line);
    if strings.len() < 2 {
        return None;
    }

    let separator = line.get(strings[0].end..strings[1].start)?;
    if !separator.contains(" at ") {
        return None;
    }

    Some(MavenNamedRepository {
        id: strings[0].value.to_owned(),
        url: strings[1].value.to_owned(),
    })
}

fn parse_sbt_dependency_line(
    text: &str,
    line: &str,
    line_offset: usize,
    group: &str,
    scala_binary_version: Option<&str>,
) -> Vec<Dependency> {
    let strings = quoted_strings(line);
    let mut dependencies = vec![];
    let mut index = 0;

    while index + 2 < strings.len() {
        if let Some(dependency) = sbt_dependency_from_strings(
            SbtDependencyLine {
                text,
                line,
                line_offset,
                dependency_group: group,
                scala_binary_version,
            },
            SbtDependencyStrings {
                group: strings[index],
                artifact: strings[index + 1],
                version: strings[index + 2],
            },
        ) {
            dependencies.push(dependency);
            index += 3;
        } else {
            index += 1;
        }
    }

    dependencies
}

struct SbtDependencyLine<'a> {
    text: &'a str,
    line: &'a str,
    line_offset: usize,
    dependency_group: &'a str,
    scala_binary_version: Option<&'a str>,
}

struct SbtDependencyStrings<'a> {
    group: QuotedString<'a>,
    artifact: QuotedString<'a>,
    version: QuotedString<'a>,
}

fn sbt_dependency_from_strings(
    line_context: SbtDependencyLine<'_>,
    strings: SbtDependencyStrings<'_>,
) -> Option<Dependency> {
    let group_string = strings.group;
    let artifact_string = strings.artifact;
    let version_string = strings.version;
    let group = group_string.value;
    let artifact = artifact_string.value;
    let version = version_string.value;
    let operator_text = line_context
        .line
        .get(group_string.end..artifact_string.start)?;
    let scala_cross = operator_text.contains("%%");
    if !operator_text.contains('%')
        || !line_context
            .line
            .get(artifact_string.end..version_string.start)?
            .contains('%')
    {
        return None;
    }

    let hosted_url = if sbt_dependency_has_explicit_url(line_context.line, version_string.end) {
        Some("url".to_owned())
    } else {
        None
    };

    let (name, hosted_url) = if scala_cross {
        match line_context.scala_binary_version {
            Some(scala_binary_version) => (
                format!("{group}:{artifact}_{scala_binary_version}"),
                hosted_url,
            ),
            None => (
                format!("{group}:{artifact}"),
                hosted_url.or_else(|| Some("scala-binary-version".to_owned())),
            ),
        }
    } else {
        (format!("{group}:{artifact}"), hosted_url)
    };

    Some(Dependency {
        name,
        requirement: version.to_owned(),
        ecosystem: Maven,
        group: line_context.dependency_group.to_owned(),
        hosted_url,
        hosted_name: scala_cross.then(|| artifact.to_owned()),
        range: offset_range(
            line_context.text,
            line_context.line_offset + group_string.start,
            line_context.line_offset + version_string.end,
        ),
        requirement_range: offset_range(
            line_context.text,
            line_context.line_offset + version_string.content_start,
            line_context.line_offset + version_string.content_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn sbt_dependency_group(line: &str) -> Option<&'static str> {
    let first_string = line.find('"').unwrap_or(line.len());
    let prefix = line.get(..first_string)?;
    if prefix.contains("libraryDependencies") {
        Some("libraryDependencies")
    } else if prefix.contains("dependencyOverrides") {
        Some("dependencyOverrides")
    } else {
        None
    }
}

fn sbt_dependency_has_explicit_url(line: &str, after_version: usize) -> bool {
    line.get(after_version..)
        .is_some_and(|tail| tail.contains(" from "))
}

fn paren_delta(line: &str) -> i32 {
    line.chars().fold(0, |balance, character| match character {
        '(' => balance + 1,
        ')' => balance - 1,
        _ => balance,
    })
}

fn scala_binary_version(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim_start();
        let first_string = trimmed.find('"').unwrap_or(trimmed.len());
        let setting = trimmed.get(..first_string)?;
        if !setting.contains("scalaVersion") {
            continue;
        }
        let strings = quoted_strings(line);
        let version = strings.first()?.value;
        let mut parts = version.split('.');
        let major = parts.next()?;
        let minor = parts.next()?;
        return Some(format!("{major}.{minor}"));
    }

    None
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

    while let Some(relative_start) = line.get(search_start..).and_then(|tail| tail.find('"')) {
        let start = search_start + relative_start;
        let content_start = start + 1;
        let Some(relative_end) = line.get(content_start..).and_then(|tail| tail.find('"')) else {
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
