use crate::positions::offset_range;
use crate::support;
use versionlens_model::Ecosystem::Opam;
use versionlens_model::{Dependency, Range};

pub(crate) fn parse_opam(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let name = quoted_field(text, "name");

    if let (Some(name), Some(version)) = (name.as_ref(), quoted_field(text, "version")) {
        dependencies.push(version_dependency(
            name.value.as_str(),
            version.value.as_str(),
            "version",
            offset_range(text, version.field_start, version.value_end + 1),
            offset_range(text, version.value_start, version.value_end),
        ));
    }

    for group in ["depends", "depopts", "conflicts"] {
        dependencies.extend(parse_opam_list(text, group));
    }

    dependencies
}

pub(crate) fn version_dependency(
    name: &str,
    requirement: &str,
    group: &str,
    range: Range,
    requirement_range: Range,
) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Opam,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range,
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

struct QuotedField {
    value: String,
    field_start: usize,
    value_start: usize,
    value_end: usize,
}

fn quoted_field(text: &str, field: &str) -> Option<QuotedField> {
    let pattern = format!("{field}:");
    for (line_start, line) in support::line_offsets(text) {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let Some(after_field) = trimmed.strip_prefix(&pattern) else {
            continue;
        };
        let after_field_offset = line_start + leading + pattern.len();
        let quote_offset = after_field.find('"')?;
        let value_start = after_field_offset + quote_offset + 1;
        let value_text = text.get(value_start..)?;
        let value_len = value_text.find('"')?;
        let value_end = value_start + value_len;
        return Some(QuotedField {
            value: text.get(value_start..value_end)?.to_owned(),
            field_start: line_start + leading,
            value_start,
            value_end,
        });
    }

    None
}

fn parse_opam_list(text: &str, group: &str) -> Vec<Dependency> {
    let Some((list_start, list_end)) = list_value_span(text, group) else {
        return vec![];
    };

    let mut dependencies = vec![];
    let mut cursor = list_start;
    while cursor < list_end {
        let Some(relative_quote) = text[cursor..list_end].find('"') else {
            break;
        };
        let name_quote_start = cursor + relative_quote;
        let name_start = name_quote_start + 1;
        let Some(relative_name_end) = text[name_start..list_end].find('"') else {
            break;
        };
        let name_end = name_start + relative_name_end;
        let Some(name) = text
            .get(name_start..name_end)
            .filter(|name| !name.is_empty())
        else {
            cursor = name_end + 1;
            continue;
        };

        let after_name = support::skip_ascii_whitespace_until(text, name_end + 1, list_end);
        let (
            requirement,
            requirement_start,
            requirement_end,
            requirement_prefix,
            requirement_suffix,
            entry_end,
        ) = if text.as_bytes().get(after_name) == Some(&b'{') {
            let formula_end =
                matching_delimiter(text, after_name, list_end, b'{', b'}').unwrap_or(after_name);
            if let Some(constraint) = first_version_constraint(text, after_name + 1, formula_end) {
                (
                    constraint.requirement,
                    constraint.range_start,
                    constraint.range_end,
                    constraint.prefix,
                    constraint.suffix,
                    formula_end + 1,
                )
            } else {
                (
                    "latest".to_owned(),
                    name_end + 1,
                    name_end + 1,
                    " {>= \"".to_owned(),
                    "\"}".to_owned(),
                    formula_end + 1,
                )
            }
        } else {
            (
                "latest".to_owned(),
                name_end + 1,
                name_end + 1,
                " {>= \"".to_owned(),
                "\"}".to_owned(),
                name_end + 1,
            )
        };

        dependencies.push(Dependency {
            name: name.to_owned(),
            requirement,
            ecosystem: Opam,
            group: group.to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, name_quote_start, entry_end),
            requirement_range: offset_range(text, requirement_start, requirement_end),
            requirement_prefix,
            requirement_suffix,
            canonical_reference: None,
        });
        cursor = entry_end;
    }

    dependencies
}

fn list_value_span(text: &str, field: &str) -> Option<(usize, usize)> {
    let (line_start, _, after_field, value_offset) = support::field_line(text, field, false)?;
    let bracket_relative = after_field.find('[')?;
    let list_start = line_start + value_offset + bracket_relative + 1;
    let list_end = matching_delimiter(text, list_start - 1, text.len(), b'[', b']')?;
    Some((list_start, list_end))
}

struct OpamConstraint {
    requirement: String,
    range_start: usize,
    range_end: usize,
    prefix: String,
    suffix: String,
}

fn first_version_constraint(text: &str, start: usize, end: usize) -> Option<OpamConstraint> {
    let constraint =
        support::first_constraint(text, start, end, &[">=", "<=", "!=", "=", "<", ">"], true)?;
    let version = text.get(constraint.version_start..constraint.version_end)?;
    Some(OpamConstraint {
        requirement: format!("{} {version}", constraint.operator),
        range_start: constraint.operator_start,
        range_end: constraint.range_end,
        prefix: format!("{} \"", constraint.operator),
        suffix: "\"".to_owned(),
    })
}

fn matching_delimiter(
    text: &str,
    open_offset: usize,
    limit: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(open_offset) != Some(&open) {
        return None;
    }

    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut offset = open_offset;
    while offset < limit {
        let byte = bytes[offset];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            offset += 1;
            continue;
        }

        if byte == b'"' {
            in_string = true;
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(offset);
            }
        }
        offset += 1;
    }

    None
}
