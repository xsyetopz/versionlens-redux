use crate::positions::offset_range;
use crate::requirement_range::operator_requirement_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::LuaRocks;

const DEPENDENCY_GROUPS: &[&str] = &["dependencies", "build_dependencies", "test_dependencies"];

pub(crate) fn parse_luarocks_rockspec(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut active_group: Option<&str> = None;
    let mut table_depth = 0usize;
    let mut offset = 0usize;

    for line in text.lines() {
        let trimmed = line.trim_start();
        if active_group.is_none()
            && let Some((group, relative)) = dependency_group_assignment(trimmed)
        {
            active_group = Some(group);
            table_depth = brace_delta(&trimmed[relative..], 0);
        }

        if let Some(group) = active_group {
            dependencies.extend(parse_dependency_strings_in_line(text, line, offset, group));
            if !opens_dependency_table(trimmed) {
                table_depth = brace_delta(trimmed, table_depth);
            }
            if table_depth == 0 {
                active_group = None;
            }
        }

        offset += line.len() + 1;
    }

    dependencies
}

fn dependency_group_assignment(line: &str) -> Option<(&'static str, usize)> {
    for group in DEPENDENCY_GROUPS {
        let Some(rest) = line.strip_prefix(group) else {
            continue;
        };
        let rest = rest.trim_start();
        if rest.starts_with('=') {
            return Some((group, line.find('{')?));
        }
    }
    None
}

fn opens_dependency_table(line: &str) -> bool {
    DEPENDENCY_GROUPS.iter().any(|group| {
        line.strip_prefix(group)
            .is_some_and(|rest| rest.contains('{'))
    })
}

fn brace_delta(line: &str, mut depth: usize) -> usize {
    for ch in line.chars() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn parse_dependency_strings_in_line(
    text: &str,
    line: &str,
    line_offset: usize,
    group: &str,
) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut cursor = 0usize;
    while let Some(relative_quote) = line[cursor..].find('"') {
        let quote = cursor + relative_quote;
        let value_start = quote + 1;
        let Some(value_end_relative) = line[value_start..].find('"') else {
            break;
        };
        let value_end = value_start + value_end_relative;
        if let Some(dependency) = parse_dependency_string(
            text,
            line_offset + value_start,
            &line[value_start..value_end],
            group,
        ) {
            dependencies.push(dependency);
        }
        cursor = value_end + 1;
    }
    dependencies
}

fn parse_dependency_string(
    text: &str,
    value_start: usize,
    value: &str,
    group: &str,
) -> Option<Dependency> {
    let trimmed_start = value.len() - value.trim_start().len();
    let trimmed = &value[trimmed_start..];
    let name_len = trimmed.find(crate::is_whitespace).unwrap_or(trimmed.len());
    let name = &trimmed[..name_len];
    if name.is_empty() {
        return None;
    }
    let requirement_start = trimmed_start
        + name_len
        + value[trimmed_start + name_len..]
            .find(|ch: char| !ch.is_whitespace())
            .unwrap_or(0);
    let requirement = if requirement_start > trimmed_start + name_len {
        value[requirement_start..].trim().to_owned()
    } else {
        "".to_owned()
    };
    let range = operator_requirement_range(&requirement, &["==", "~=", "<=", ">=", "~>", "<", ">"]);
    let name_start = value_start + trimmed_start;
    let requirement_absolute_start = value_start + requirement_start + range.start;
    let requirement_absolute_end = value_start + requirement_start + range.end;

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: LuaRocks,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, name_start, name_start + name_len),
        requirement_range: offset_range(text, requirement_absolute_start, requirement_absolute_end),
        requirement_prefix: range.prefix,
        requirement_suffix: "".to_owned(),
    })
}
