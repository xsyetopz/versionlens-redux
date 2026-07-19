use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Hackage;

type ParsedHackageDependency = Option<Dependency>;
type HackageDependencies = Vec<Dependency>;
type TextSpans = Vec<TextSpan>;

const CABAL_DEPENDENCY_FIELDS: &[&str] = &["build-depends", "setup-depends", "build-tool-depends"];

pub(crate) fn parse_cabal(text: &str) -> HackageDependencies {
    let mut dependencies = vec![];
    if let (Some(name), Some(version)) = (field_value(text, "name"), field_value(text, "version")) {
        dependencies.push(Dependency {
            name: name.value.as_str().to_owned(),
            requirement: version.value.as_str().to_owned(),
            ecosystem: Hackage,
            group: "version".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, version.field_start, version.value_end),
            requirement_range: offset_range(text, version.value_start, version.value_end),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }

    for field in CABAL_DEPENDENCY_FIELDS {
        dependencies.extend(parse_cabal_field_dependencies(text, field));
    }

    dependencies
}

pub(crate) fn parse_cabal_project(text: &str) -> HackageDependencies {
    parse_cabal_field_dependencies(text, "constraints")
}

pub(crate) fn parse_stack_yaml(text: &str) -> HackageDependencies {
    let mut dependencies = vec![];
    if let Some(resolver) = stack_resolver_dependency(text) {
        dependencies.push(resolver);
    }

    let Some((start, end)) = yaml_list_span(text, "extra-deps") else {
        return dependencies;
    };

    let mut offset = start;
    while offset < end {
        let line_end = text[offset..end]
            .find('\n')
            .map(|index| offset + index)
            .unwrap_or(end);
        let line = &text[offset..line_end];
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        if let Some(value) = trimmed.strip_prefix("- ") {
            let value_start = offset + leading + 2;
            if let Some(dependency) = parse_stack_extra_dep(text, value, value_start) {
                dependencies.push(dependency);
            }
        }
        offset = line_end.saturating_add(1);
    }

    dependencies
}

fn stack_resolver_dependency(text: &str) -> ParsedHackageDependency {
    let resolver = field_value(text, "resolver")?;
    let value = resolver.value.trim();
    if value.is_empty() {
        return None;
    }

    if let Some(version) = value.strip_prefix("lts-") {
        return stackage_resolver_dependency(text, &resolver, "stackage-lts", version, value);
    }
    if let Some(version) = value.strip_prefix("nightly-") {
        return stackage_resolver_dependency(text, &resolver, "stackage-nightly", version, value);
    }

    Some(Dependency {
        name: value.to_owned(),
        requirement: value.to_owned(),
        ecosystem: Hackage,
        group: "resolver".to_owned(),
        hosted_url: Some("stack-resolver".to_owned()),
        hosted_name: None,
        range: offset_range(text, resolver.field_start, resolver.value_end),
        requirement_range: offset_range(text, resolver.value_start, resolver.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn stackage_resolver_dependency(
    text: &str,
    resolver: &FieldValue,
    name: &str,
    version: &str,
    value: &str,
) -> ParsedHackageDependency {
    if version.is_empty() {
        return None;
    }
    let version_start = resolver.value_start + value.len() - version.len();
    Some(Dependency {
        name: name.to_owned(),
        requirement: version.to_owned(),
        ecosystem: Hackage,
        group: "resolver".to_owned(),
        hosted_url: Some("stackage".to_owned()),
        hosted_name: None,
        range: offset_range(text, resolver.field_start, resolver.value_end),
        requirement_range: offset_range(text, version_start, resolver.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

struct FieldValue {
    value: String,
    field_start: usize,
    value_start: usize,
    value_end: usize,
}

fn field_value(text: &str, field: &str) -> Option<FieldValue> {
    let pattern = format!("{field}:");
    for (line_start, line) in line_offsets(text) {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        if leading != 0 {
            continue;
        }
        let Some(value) = trimmed.strip_prefix(&pattern) else {
            continue;
        };
        let value_start = line_start + pattern.len() + value.len() - value.trim_start().len();
        let value_end = line_start + line.trim_end().len();
        return Some(FieldValue {
            value: text[value_start..value_end].trim().to_owned(),
            field_start: line_start,
            value_start,
            value_end,
        });
    }

    None
}

fn parse_cabal_field_dependencies(text: &str, field: &str) -> HackageDependencies {
    let mut dependencies = vec![];
    for span in cabal_field_value_spans(text, field) {
        for entry in split_cabal_dependencies(text, span.start, span.end) {
            if let Some(dependency) = parse_cabal_dependency_entry(text, field, entry) {
                dependencies.push(dependency);
            }
        }
    }

    dependencies
}

#[derive(Clone, Copy)]
struct TextSpan {
    start: usize,
    end: usize,
}

fn cabal_field_value_spans(text: &str, field: &str) -> TextSpans {
    let pattern = format!("{field}:");
    let lines = line_offsets(text).collect::<Vec<_>>();
    let mut spans = vec![];
    let mut index = 0usize;
    while index < lines.len() {
        let (line_start, line) = lines[index];
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let Some(after_field) = trimmed.strip_prefix(&pattern) else {
            index += 1;
            continue;
        };
        let value_start = line_start + leading + pattern.len();
        let mut value_end = line_start + line.len();
        let field_indent = leading;
        index += 1;
        while let Some((next_start, next_line)) = lines.get(index).copied() {
            if next_line.trim().is_empty() {
                value_end = next_start + next_line.len();
                index += 1;
                continue;
            }
            let next_trimmed = next_line.trim_start();
            let next_indent = next_line.len() - next_trimmed.len();
            if next_indent <= field_indent && next_trimmed.contains(':') {
                break;
            }
            value_end = next_start + next_line.len();
            index += 1;
        }
        let start = value_start + after_field.len() - after_field.trim_start().len();
        spans.push(TextSpan {
            start,
            end: value_end,
        });
    }

    spans
}

fn split_cabal_dependencies(text: &str, start: usize, end: usize) -> TextSpans {
    let mut entries = vec![];
    let mut entry_start = start;
    let mut depth = 0usize;
    let mut offset = start;
    while offset < end {
        let byte = text.as_bytes()[offset];
        match byte {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                push_trimmed_span(text, entry_start, offset, &mut entries);
                entry_start = offset + 1;
            }
            _ => {}
        }
        offset += 1;
    }
    push_trimmed_span(text, entry_start, end, &mut entries);
    entries
}

fn push_trimmed_span(text: &str, start: usize, end: usize, out: &mut Vec<TextSpan>) {
    let start = skip_whitespace_and_commas(text, start, end);
    let end = trim_ascii_end(text, start, end);
    if start < end {
        out.push(TextSpan { start, end });
    }
}

fn parse_cabal_dependency_entry(
    text: &str,
    group: &str,
    entry: TextSpan,
) -> ParsedHackageDependency {
    let raw = text.get(entry.start..entry.end)?.trim();
    let name_len = raw
        .find(|ch: char| {
            ch.is_ascii_whitespace()
                || ch == ','
                || matches!(ch, '<' | '>' | '=' | '^' | '(' | '[' | '{')
        })
        .unwrap_or(raw.len());
    let raw_name = raw.get(..name_len)?.trim();
    let name = raw_name.split(':').next().unwrap_or(raw_name).trim();
    if name.is_empty() {
        return None;
    }

    let name_start = entry.start + raw.find(raw_name)?;
    let name_end = name_start + name.len();
    let constraint_start = name_start + name_len;
    let constraint = first_cabal_constraint(text, constraint_start, entry.end);
    let (requirement, requirement_start, requirement_end, requirement_prefix, requirement_suffix) =
        if let Some(constraint) = constraint {
            (
                constraint.requirement,
                constraint.range_start,
                constraint.range_end,
                constraint.prefix,
                "".to_owned(),
            )
        } else {
            (
                "latest".to_owned(),
                name_end,
                name_end,
                " >= ".to_owned(),
                "".to_owned(),
            )
        };

    Some(Dependency {
        name: name.to_owned(),
        requirement,
        ecosystem: Hackage,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, entry.start, entry.end),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix,
        requirement_suffix,
    })
}

struct CabalConstraint {
    requirement: String,
    range_start: usize,
    range_end: usize,
    prefix: String,
}

fn first_cabal_constraint(text: &str, start: usize, end: usize) -> Option<CabalConstraint> {
    let mut cursor = start;
    while cursor < end {
        let op_start = skip_ascii_whitespace(text, cursor, end);
        let Some((operator, after_operator)) = cabal_operator(text, op_start, end) else {
            cursor = op_start + text[op_start..end].chars().next()?.len_utf8();
            continue;
        };
        let version_start = skip_ascii_whitespace(text, after_operator, end);
        let version_end = version_start
            + text[version_start..end]
                .find(|ch: char| {
                    ch.is_ascii_whitespace()
                        || ch == ','
                        || ch == '&'
                        || ch == '|'
                        || matches!(ch, ')' | ']' | '}')
                })
                .unwrap_or(end - version_start);
        if version_start == version_end {
            cursor = after_operator;
            continue;
        }
        let version = text.get(version_start..version_end)?;
        return Some(CabalConstraint {
            requirement: format!("{operator} {version}"),
            range_start: op_start,
            range_end: version_end,
            prefix: text.get(op_start..version_start)?.to_owned(),
        });
    }

    None
}

fn cabal_operator(text: &str, start: usize, end: usize) -> Option<(&str, usize)> {
    for operator in ["^>=", ">=", "<=", "==", ">", "<", "="] {
        let operator_end = start + operator.len();
        if operator_end <= end && text.get(start..operator_end) == Some(operator) {
            return Some((operator, operator_end));
        }
    }

    None
}

fn yaml_list_span(text: &str, field: &str) -> Option<(usize, usize)> {
    let pattern = format!("{field}:");
    let lines = line_offsets(text).collect::<Vec<_>>();
    for (index, (line_start, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        if trimmed != pattern {
            continue;
        }
        let start = line_start + line.len() + 1;
        let mut end = text.len();
        for (next_start, next_line) in lines.iter().copied().skip(index + 1) {
            if next_line.trim().is_empty() {
                continue;
            }
            let next_indent = next_line.len() - next_line.trim_start().len();
            if next_indent <= leading && !next_line.trim_start().starts_with('-') {
                end = next_start;
                break;
            }
        }
        return Some((start, end));
    }

    None
}

fn parse_stack_extra_dep(text: &str, value: &str, value_start: usize) -> ParsedHackageDependency {
    let value = value.trim_end();
    if let Some(url) = value.strip_prefix("git:").map(|value| value.trim()) {
        let requirement_start = value_start + value.find(url)?;
        return Some(fixed_stack_dependency(
            text,
            url,
            "git",
            requirement_start,
            requirement_start + url.len(),
        ));
    }
    if let Some(url) = value.strip_prefix("github:").map(|value| value.trim()) {
        let requirement_start = value_start + value.find(url)?;
        return Some(fixed_stack_dependency(
            text,
            url,
            "git",
            requirement_start,
            requirement_start + url.len(),
        ));
    }
    if value.starts_with("./")
        || value.starts_with("../")
        || value.starts_with('/')
        || value.contains('/')
    {
        let value_end = value_start + value.len();
        return Some(fixed_stack_dependency(
            text,
            value,
            "path",
            value_start,
            value_end,
        ));
    }

    parse_hackage_package_id(text, value, value_start)
}

fn fixed_stack_dependency(
    text: &str,
    value: &str,
    hosted_url: &str,
    value_start: usize,
    value_end: usize,
) -> Dependency {
    Dependency {
        name: value.to_owned(),
        requirement: value.to_owned(),
        ecosystem: Hackage,
        group: "extra-deps".to_owned(),
        hosted_url: Some(hosted_url.to_owned()),
        hosted_name: None,
        range: offset_range(text, value_start, value_end),
        requirement_range: offset_range(text, value_start, value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn parse_hackage_package_id(
    text: &str,
    value: &str,
    value_start: usize,
) -> ParsedHackageDependency {
    let package = value.split('@').next().unwrap_or(value);
    let split = package.rfind('-')?;
    let name = package.get(..split)?;
    let version = package.get(split + 1..)?;
    if name.is_empty() || version.is_empty() || !version.as_bytes()[0].is_ascii_digit() {
        return None;
    }
    let requirement_start = value_start + split + 1;
    let requirement_end = requirement_start + version.len();
    let suffix_start = requirement_end - value_start;
    Some(Dependency {
        name: name.to_owned(),
        requirement: version.to_owned(),
        ecosystem: Hackage,
        group: "extra-deps".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, value_start, value_start + value.len()),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: value.get(suffix_start..).unwrap_or_default().to_owned(),
    })
}

fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    text.lines().map(move |line| {
        let current = offset;
        offset += line.len() + 1;
        (current, line)
    })
}

fn skip_whitespace_and_commas(text: &str, mut offset: usize, end: usize) -> usize {
    while offset < end
        && (text.as_bytes()[offset].is_ascii_whitespace() || text.as_bytes()[offset] == b',')
    {
        offset += 1;
    }
    offset
}

fn skip_ascii_whitespace(text: &str, mut offset: usize, end: usize) -> usize {
    while offset < end && text.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
}

fn trim_ascii_end(text: &str, start: usize, mut end: usize) -> usize {
    while end > start
        && (text.as_bytes()[end - 1].is_ascii_whitespace() || text.as_bytes()[end - 1] == b',')
    {
        end -= 1;
    }
    end
}
