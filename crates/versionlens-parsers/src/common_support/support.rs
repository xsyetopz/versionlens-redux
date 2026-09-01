use crate::positions;
use jsonc_parser::ast::{Object, Value};
use jsonc_parser::parse_to_ast;
use marked_yaml::{parse_yaml, types::MarkedMappingNode};
use quick_xml::Reader as XmlReader;
use std::path::Path as StdPath;
use toml_edit::{Document as TomlDocument, TomlError};

pub(crate) struct DependencyParts {
    pub(crate) requirement: String,
    pub(crate) name_start: usize,
    pub(crate) name_end: usize,
    pub(crate) requirement_start: usize,
    pub(crate) requirement_end: usize,
}

pub(crate) struct SourcedDependency<'a> {
    pub(crate) text: &'a str,
    pub(crate) ecosystem: versionlens_model::Ecosystem,
    pub(crate) group: &'a str,
    pub(crate) name: &'a str,
    pub(crate) requirement: &'a str,
    pub(crate) hosted_url: Option<&'a str>,
    pub(crate) hosted_name: Option<&'a str>,
    pub(crate) range: std::ops::Range<usize>,
    pub(crate) requirement_range: std::ops::Range<usize>,
}

pub(crate) fn sourced_dependency(case: SourcedDependency<'_>) -> versionlens_model::Dependency {
    versionlens_model::Dependency {
        name: case.name.to_owned(),
        requirement: case.requirement.to_owned(),
        ecosystem: case.ecosystem,
        group: case.group.to_owned(),
        hosted_url: case.hosted_url.map(str::to_owned),
        hosted_name: case.hosted_name.map(str::to_owned),
        range: positions::offset_range(case.text, case.range.start, case.range.end),
        requirement_range: positions::offset_range(
            case.text,
            case.requirement_range.start,
            case.requirement_range.end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

pub(crate) fn dependency(
    text: &str,
    ecosystem: versionlens_model::Ecosystem,
    group: &str,
    name: &str,
    parts: DependencyParts,
) -> versionlens_model::Dependency {
    versionlens_model::Dependency {
        name: name.to_owned(),
        requirement: parts.requirement,
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: positions::offset_range(text, parts.name_start, parts.name_end),
        requirement_range: positions::offset_range(
            text,
            parts.requirement_start,
            parts.requirement_end,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn path(value: &str) -> &StdPath {
    value.as_ref()
}

pub(crate) fn xml_reader(text: &str) -> XmlReader<&[u8]> {
    quick_xml::Reader::from_str(text)
}

pub(crate) fn with_yaml_mapping<T>(
    text: &str,
    parse: impl FnOnce(&MarkedMappingNode) -> T,
) -> Option<T> {
    let document = parse_yaml(0, text).ok()?;
    let root = document.as_mapping()?;
    Some(parse(&root))
}

pub(crate) fn with_json_object<T>(text: &str, parse: impl FnOnce(&Object<'_>) -> T) -> Option<T> {
    let parsed = parse_to_ast(text, &default(), &default()).ok()?;
    let Value::Object(root) = parsed.value? else {
        return None;
    };
    Some(parse(&root))
}

pub(crate) fn try_with_json_object<T>(
    text: &str,
    parse: impl FnOnce(&Object<'_>) -> T,
) -> Result<Option<T>, jsonc_parser::errors::ParseError> {
    let parsed = parse_to_ast(text, &default(), &default())?;
    let Some(Value::Object(root)) = parsed.value else {
        return Ok(None);
    };
    Ok(Some(parse(&root)))
}

pub(crate) fn quoted_span(text: &str, quote: usize) -> Option<(usize, usize)> {
    let mut offset = quote + 1;
    let mut escaped = false;
    while offset < text.len() {
        let byte = *text.as_bytes().get(offset)?;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some((quote + 1, offset));
        }
        offset += 1;
    }
    None
}

pub(crate) fn skip_ascii_whitespace(text: &str, mut offset: usize) -> usize {
    while text
        .as_bytes()
        .get(offset)
        .is_some_and(|value| value.is_ascii_whitespace())
    {
        offset += 1;
    }
    offset
}

pub(crate) fn skip_ascii_whitespace_until(text: &str, mut offset: usize, end: usize) -> usize {
    while offset < end && text.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
}

pub(crate) fn dependency_path_is_selected(paths: &[&str], path: &str) -> bool {
    paths.is_empty() || paths.contains(&path)
}

pub(crate) fn append_source_line(buffer: &mut String, line: &str) {
    buffer.push('\n');
    buffer.push_str(line);
}

pub(crate) fn operator_at<'a>(
    text: &'a str,
    start: usize,
    end: usize,
    operators: &[&'a str],
) -> Option<(&'a str, usize)> {
    operators.iter().find_map(|operator| {
        let operator_end = start + operator.len();
        (operator_end <= end && text.get(start..operator_end) == Some(*operator))
            .then_some((*operator, operator_end))
    })
}

pub(crate) struct ConstraintMatch {
    pub(crate) operator: String,
    pub(crate) operator_start: usize,
    pub(crate) version_start: usize,
    pub(crate) version_end: usize,
    pub(crate) range_end: usize,
}

pub(crate) fn first_constraint(
    text: &str,
    start: usize,
    end: usize,
    operators: &[&str],
    quoted: bool,
) -> Option<ConstraintMatch> {
    let mut cursor = start;
    while cursor < end {
        let op_start = skip_ascii_whitespace_until(text, cursor, end);
        let Some((operator, after_operator)) = operator_at(text, op_start, end, operators) else {
            cursor = op_start + text[op_start..end].chars().next()?.len_utf8();
            continue;
        };
        let value_start = skip_ascii_whitespace_until(text, after_operator, end);
        let (version_start, version_end, range_end) = if quoted {
            if text.as_bytes().get(value_start) != Some(&b'"') {
                cursor = after_operator;
                continue;
            }
            let version_start = value_start + 1;
            let version_end = version_start + text[version_start..end].find('"')?;
            (version_start, version_end, version_end + 1)
        } else {
            let version_end = version_start_or_delimiter(text, value_start, end);
            (value_start, version_end, version_end)
        };
        if version_start == version_end {
            cursor = after_operator;
            continue;
        }
        return Some(ConstraintMatch {
            operator: operator.to_owned(),
            operator_start: op_start,
            version_start,
            version_end,
            range_end,
        });
    }
    None
}

fn version_start_or_delimiter(text: &str, start: usize, end: usize) -> usize {
    start
        + text[start..end]
            .find(|ch: char| {
                ch.is_ascii_whitespace()
                    || ch == ','
                    || ch == '&'
                    || ch == '|'
                    || matches!(ch, ')' | ']' | '}')
            })
            .unwrap_or(end - start)
}

pub(crate) fn quoted_value_after<'a>(source: &'a str, marker: &str) -> Option<&'a str> {
    let start = source.find(marker)? + marker.len();
    let value = source.get(start..)?.trim_start();
    let value = value.strip_prefix('"')?;
    let end = value.find('"')?;
    value.get(..end)
}

#[derive(Clone, Copy)]
pub(crate) struct QuotedString<'a> {
    pub(crate) value: &'a str,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) content_start: usize,
    pub(crate) content_end: usize,
}

pub(crate) fn quoted_strings<'a>(line: &'a str, quotes: &[char]) -> Vec<QuotedString<'a>> {
    let mut strings = vec![];
    let mut search_start = 0;
    while let Some(relative_start) = line.get(search_start..).and_then(|tail| tail.find(quotes)) {
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

pub(crate) fn string_content_start(start: usize, end: usize) -> usize {
    start + usize::from(end > start)
}
pub(crate) fn string_content_end(start: usize, end: usize) -> usize {
    end.saturating_sub(usize::from(end > start))
}
pub(crate) fn string_content_span(start: usize, end: usize) -> std::ops::Range<usize> {
    string_content_start(start, end)..string_content_end(start, end)
}

pub(crate) fn balanced_brace_spans(
    line: &str,
    scan_start: usize,
    require_keyword_prefix: bool,
) -> Vec<(usize, usize)> {
    let mut spans = vec![];
    let mut depth = 0usize;
    let mut brace_start = None;
    for (relative_index, ch) in line[scan_start..].char_indices() {
        let index = scan_start + relative_index;
        match ch {
            '{' => {
                if depth == 0 {
                    brace_start = Some(index);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0
                    && let Some(start) = brace_start.take()
                    && (!require_keyword_prefix || line.get(start..start + 2) == Some("{:"))
                {
                    spans.push((start, index + ch.len_utf8()));
                }
            }
            _ => {}
        }
    }
    spans
}

pub(crate) fn field_line<'a>(
    text: &'a str,
    field: &str,
    require_unindented: bool,
) -> Option<(usize, &'a str, &'a str, usize)> {
    let pattern = format!("{field}:");
    for (line_start, line) in line_offsets(text) {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        if require_unindented && leading != 0 {
            continue;
        }
        if let Some(value) = trimmed.strip_prefix(&pattern) {
            return Some((line_start, line, value, leading + pattern.len()));
        }
    }
    None
}

pub(crate) fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    text.lines().map(move |line| {
        let current = offset;
        offset += line.len() + 1;
        (current, line)
    })
}

pub(crate) fn valid_package_alias_name(spec: &str) -> bool {
    if spec.is_empty() || spec.contains(':') {
        return false;
    }
    if let Some(scoped) = spec.strip_prefix('@') {
        return scoped.split_once('/').is_some_and(|(scope, name)| {
            !scope.is_empty() && !name.is_empty() && !name.contains('/')
        });
    }
    !spec.contains('/')
}

pub(crate) fn is_whitespace(value: char) -> bool {
    value.is_whitespace()
}
pub(crate) fn parse_toml_document(text: &str) -> Result<TomlDocument<String>, TomlError> {
    text.parse()
}
pub(crate) fn string_from_utf8_lossy(bytes: &[u8]) -> String {
    <String>::from_utf8_lossy(bytes).into_owned()
}

pub(crate) fn requirement_parts<'a>(
    requirement: &'a str,
    value_start: usize,
    operators: &[&str],
) -> (&'a str, usize, String) {
    let trimmed = requirement.trim_start();
    let leading = requirement.len() - trimmed.len();
    for operator in operators {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            let rest_trimmed = rest.trim_start();
            if rest_trimmed.is_empty() {
                break;
            }
            let whitespace = rest.len() - rest_trimmed.len();
            return (
                trimmed,
                value_start + leading + operator.len() + whitespace,
                (*operator).to_owned(),
            );
        }
    }
    (trimmed, value_start + leading, String::new())
}

pub(crate) fn balanced_delimited_end(
    text: &str,
    start: usize,
    opening: u8,
    closing: u8,
) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in bytes.iter().copied().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        if byte == b'"' {
            in_string = true;
        } else if byte == opening {
            depth += 1;
        } else if byte == closing {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(offset + 1);
            }
        }
    }
    None
}

#[cfg(test)]
pub(crate) mod tests;
