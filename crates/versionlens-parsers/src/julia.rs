use std::collections::BTreeMap;

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Julia;

type ParsedStringEntry = Option<StringEntry>;

pub(crate) fn parse_julia_project(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let name = root_string_field(text, "name");
    let version = root_string_field(text, "version");
    if let (Some(name), Some(version)) = (name.as_ref(), version.as_ref()) {
        dependencies.push(Dependency {
            name: name.value.as_str().to_owned(),
            requirement: version.value.as_str().to_owned(),
            ecosystem: Julia,
            group: "version".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, version.line_start, version.line_end),
            requirement_range: offset_range(text, version.value_start, version.value_end),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }

    let deps = table_string_entries(text, "deps");
    let compat = table_string_entries(text, "compat");
    let sources = table_source_entries(text, "sources");
    let mut names = deps
        .iter()
        .map(|entry| entry.key.as_str().to_owned())
        .collect::<Vec<_>>();
    for entry in &compat {
        if entry.key != "julia" && !names.contains(&entry.key) {
            names.push(entry.key.as_str().to_owned());
        }
    }

    for package in names {
        if let Some(source) = sources.get(&package) {
            dependencies.push(source_dependency(text, "sources", &package, source));
            continue;
        }

        if let Some(entry) = compat.iter().find(|entry| entry.key == package) {
            dependencies.push(string_entry_dependency(text, "compat", entry));
        } else if let Some(entry) = deps.iter().find(|entry| entry.key == package) {
            dependencies.push(Dependency {
                name: entry.key.as_str().to_owned(),
                requirement: "latest".to_owned(),
                ecosystem: Julia,
                group: "deps".to_owned(),
                hosted_url: None,
                hosted_name: Some(entry.value.as_str().to_owned()),
                range: offset_range(text, entry.line_start, entry.line_end),
                requirement_range: offset_range(text, entry.value_end, entry.value_end),
                requirement_prefix: " # compat ".to_owned(),
                requirement_suffix: "".to_owned(),
            });
        }
    }

    if let Some(entry) = compat.iter().find(|entry| entry.key == "julia") {
        dependencies.push(string_entry_dependency(text, "compat", entry));
    }

    dependencies
}

pub(crate) fn parse_julia_manifest(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let lines = line_offsets(text).collect::<Vec<_>>();
    let mut index = 0usize;
    while index < lines.len() {
        let (line_start, line) = lines[index];
        let trimmed = line.trim();
        let Some(name) = trimmed
            .strip_prefix("[[deps.")
            .and_then(|value| value.strip_suffix("]]"))
        else {
            index += 1;
            continue;
        };
        let block_start = line_start;
        let mut block_end = text.len();
        let mut version = None;
        let mut path = None;
        let mut repo_url = None;
        index += 1;
        while let Some((next_start, next_line)) = lines.get(index).copied() {
            if next_line.trim_start().starts_with("[[deps.") {
                block_end = next_start;
                break;
            }
            if version.is_none() {
                version = line_string_assignment(next_start, next_line, "version");
            }
            if path.is_none() {
                path = line_string_assignment(next_start, next_line, "path");
            }
            if repo_url.is_none() {
                repo_url = line_string_assignment(next_start, next_line, "repo-url");
            }
            index += 1;
        }

        let source = path
            .as_ref()
            .map(|entry| ("path", entry))
            .or_else(|| repo_url.as_ref().map(|entry| ("git", entry)));
        if let Some((source_kind, source)) = source {
            dependencies.push(Dependency {
                name: name.to_owned(),
                requirement: source.value.as_str().to_owned(),
                ecosystem: Julia,
                group: "deps".to_owned(),
                hosted_url: Some(source_kind.to_owned()),
                hosted_name: None,
                range: offset_range(text, block_start, block_end),
                requirement_range: offset_range(text, source.value_start, source.value_end),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
            });
        } else if let Some(version) = version {
            dependencies.push(Dependency {
                name: name.to_owned(),
                requirement: version.value.as_str().to_owned(),
                ecosystem: Julia,
                group: "deps".to_owned(),
                hosted_url: None,
                hosted_name: None,
                range: offset_range(text, block_start, block_end),
                requirement_range: offset_range(text, version.value_start, version.value_end),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
            });
        }
    }

    dependencies
}

#[derive(Clone)]
struct StringEntry {
    key: String,
    value: String,
    line_start: usize,
    line_end: usize,
    value_start: usize,
    value_end: usize,
}

#[derive(Clone)]
struct SourceEntry {
    value: String,
    source: &'static str,
    line_start: usize,
    line_end: usize,
    value_start: usize,
    value_end: usize,
}

fn string_entry_dependency(text: &str, group: &str, entry: &StringEntry) -> Dependency {
    Dependency {
        name: entry.key.as_str().to_owned(),
        requirement: entry.value.as_str().to_owned(),
        ecosystem: Julia,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, entry.line_start, entry.line_end),
        requirement_range: offset_range(text, entry.value_start, entry.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn source_dependency(text: &str, group: &str, package: &str, source: &SourceEntry) -> Dependency {
    Dependency {
        name: package.to_owned(),
        requirement: source.value.as_str().to_owned(),
        ecosystem: Julia,
        group: group.to_owned(),
        hosted_url: Some(source.source.to_owned()),
        hosted_name: None,
        range: offset_range(text, source.line_start, source.line_end),
        requirement_range: offset_range(text, source.value_start, source.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn root_string_field(text: &str, field: &str) -> ParsedStringEntry {
    for (line_start, line) in line_offsets(text) {
        if line.trim_start().starts_with('[') {
            return None;
        }
        if let Some(entry) = line_string_assignment(line_start, line, field) {
            return Some(entry);
        }
    }

    None
}

fn table_string_entries(text: &str, table: &str) -> Vec<StringEntry> {
    let Some((start, end)) = table_span(text, table) else {
        return vec![];
    };
    line_offsets(text)
        .filter(|(line_start, _)| *line_start >= start && *line_start < end)
        .filter_map(|(line_start, line)| line_string_entry(line_start, line))
        .collect()
}

fn table_source_entries(text: &str, table: &str) -> BTreeMap<String, SourceEntry> {
    let Some((start, end)) = table_span(text, table) else {
        return crate::default();
    };

    let mut sources: BTreeMap<String, SourceEntry> = crate::default();
    for (line_start, line) in line_offsets(text) {
        if line_start < start || line_start >= end {
            continue;
        }
        let Some((key, value_start)) = assignment_key_and_value_start(line_start, line) else {
            continue;
        };
        let Some((source, value, value_offset)) =
            inline_source_value(line, value_start - line_start)
        else {
            continue;
        };
        sources.insert(
            key,
            SourceEntry {
                value: value.to_owned(),
                source,
                line_start,
                line_end: line_start + line.len(),
                value_start: line_start + value_offset,
                value_end: line_start + value_offset + value.len(),
            },
        );
    }

    sources
}

fn inline_source_value(
    line: &str,
    value_start_in_line: usize,
) -> Option<(&'static str, &str, usize)> {
    let value = line.get(value_start_in_line..)?;
    quoted_inline_key(value, "path")
        .map(|(value, offset)| ("path", value, value_start_in_line + offset))
        .or_else(|| {
            quoted_inline_key(value, "url")
                .map(|(value, offset)| ("git", value, value_start_in_line + offset))
        })
}

fn quoted_inline_key<'a>(value: &'a str, key: &str) -> Option<(&'a str, usize)> {
    let key_at = value.find(key)?;
    let after_key = value.get(key_at + key.len()..)?;
    let quote_relative = after_key.find('"')?;
    let start = key_at + key.len() + quote_relative + 1;
    let end = start + value.get(start..)?.find('"')?;
    Some((value.get(start..end)?, start))
}

fn table_span(text: &str, table: &str) -> Option<(usize, usize)> {
    let header = format!("[{table}]");
    let lines = line_offsets(text).collect::<Vec<_>>();
    for (index, (line_start, line)) in lines.iter().copied().enumerate() {
        if line.trim() != header {
            continue;
        }
        let start = line_start + line.len() + 1;
        let end = lines
            .iter()
            .copied()
            .skip(index + 1)
            .find(|(_, line)| line.trim_start().starts_with('['))
            .map(|(next_start, _)| next_start)
            .unwrap_or(text.len());
        return Some((start, end));
    }

    None
}

fn line_string_entry(line_start: usize, line: &str) -> ParsedStringEntry {
    let (key, value_start) = assignment_key_and_value_start(line_start, line)?;
    let value = quoted_value(line, value_start - line_start)?;
    Some(StringEntry {
        key,
        value: value.0.to_owned(),
        line_start,
        line_end: line_start + line.len(),
        value_start: line_start + value.1,
        value_end: line_start + value.1 + value.0.len(),
    })
}

fn line_string_assignment(line_start: usize, line: &str, key: &str) -> ParsedStringEntry {
    let (candidate, value_start) = assignment_key_and_value_start(line_start, line)?;
    if candidate != key {
        return None;
    }
    let value = quoted_value(line, value_start - line_start)?;
    Some(StringEntry {
        key: candidate,
        value: value.0.to_owned(),
        line_start,
        line_end: line_start + line.len(),
        value_start: line_start + value.1,
        value_end: line_start + value.1 + value.0.len(),
    })
}

fn assignment_key_and_value_start(line_start: usize, line: &str) -> Option<(String, usize)> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('[') {
        return None;
    }
    let leading = line.len() - trimmed.len();
    let equals = trimmed.find('=')?;
    let key = trimmed.get(..equals)?.trim().trim_matches('"');
    if key.is_empty() {
        return None;
    }
    Some((key.to_owned(), line_start + leading + equals + 1))
}

fn quoted_value(line: &str, start: usize) -> Option<(&str, usize)> {
    let rest = line.get(start..)?;
    let quote = rest.find('"')?;
    let value_start = start + quote + 1;
    let value_end = value_start + line.get(value_start..)?.find('"')?;
    Some((line.get(value_start..value_end)?, value_start))
}

fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    text.lines().map(move |line| {
        let current = offset;
        offset += line.len() + 1;
        (current, line)
    })
}
