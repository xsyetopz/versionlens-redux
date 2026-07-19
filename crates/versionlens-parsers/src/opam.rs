use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Opam;

pub(crate) fn parse_opam(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let name = quoted_field(text, "name");

    if let (Some(name), Some(version)) = (name.as_ref(), quoted_field(text, "version")) {
        dependencies.push(Dependency {
            name: name.value.as_str().to_owned(),
            requirement: version.value.as_str().to_owned(),
            ecosystem: Opam,
            group: "version".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, version.field_start, version.value_end + 1),
            requirement_range: offset_range(text, version.value_start, version.value_end),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }

    for group in ["depends", "depopts", "conflicts"] {
        dependencies.extend(parse_opam_list(text, group));
    }

    dependencies
}

struct QuotedField {
    value: String,
    field_start: usize,
    value_start: usize,
    value_end: usize,
}

fn quoted_field(text: &str, field: &str) -> Option<QuotedField> {
    let pattern = format!("{field}:");
    for (line_start, line) in line_offsets(text) {
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

fn line_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    text.lines().map(move |line| {
        let current = offset;
        offset += line.len() + 1;
        (current, line)
    })
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

        let after_name = skip_ascii_whitespace(text, name_end + 1, list_end);
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
        });
        cursor = entry_end;
    }

    dependencies
}

fn list_value_span(text: &str, field: &str) -> Option<(usize, usize)> {
    let pattern = format!("{field}:");
    for (line_start, line) in line_offsets(text) {
        let trimmed = line.trim_start();
        let leading = line.len() - trimmed.len();
        let Some(after_field) = trimmed.strip_prefix(&pattern) else {
            continue;
        };
        let after_field_offset = line_start + leading + pattern.len();
        let bracket_relative = after_field.find('[')?;
        let list_start = after_field_offset + bracket_relative + 1;
        let list_end = matching_delimiter(text, list_start - 1, text.len(), b'[', b']')?;
        return Some((list_start, list_end));
    }

    None
}

struct OpamConstraint {
    requirement: String,
    range_start: usize,
    range_end: usize,
    prefix: String,
    suffix: String,
}

fn first_version_constraint(text: &str, start: usize, end: usize) -> Option<OpamConstraint> {
    let mut cursor = start;
    while cursor < end {
        let op_start = skip_ascii_whitespace(text, cursor, end);
        let Some((operator, after_operator)) = opam_operator(text, op_start, end) else {
            cursor = op_start + text[op_start..end].chars().next()?.len_utf8();
            continue;
        };
        let quote = skip_ascii_whitespace(text, after_operator, end);
        if text.as_bytes().get(quote) != Some(&b'"') {
            cursor = after_operator;
            continue;
        }
        let version_start = quote + 1;
        let version_end = version_start + text[version_start..end].find('"')?;
        let version = text.get(version_start..version_end)?;
        return Some(OpamConstraint {
            requirement: format!("{operator} {version}"),
            range_start: op_start,
            range_end: version_end + 1,
            prefix: format!("{operator} \""),
            suffix: "\"".to_owned(),
        });
    }

    None
}

fn opam_operator(text: &str, start: usize, end: usize) -> Option<(&str, usize)> {
    for operator in [">=", "<=", "!=", "=", "<", ">"] {
        let operator_end = start + operator.len();
        if operator_end <= end && text.get(start..operator_end) == Some(operator) {
            return Some((operator, operator_end));
        }
    }

    None
}

fn skip_ascii_whitespace(text: &str, mut offset: usize, end: usize) -> usize {
    while offset < end && text.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
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
