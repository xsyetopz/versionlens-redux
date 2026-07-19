use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Opam;

pub(crate) fn parse_dune_project(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];

    for package in package_stanzas(text) {
        let package_body_start = package.open + "(package".len();
        let package_body_end = package.close;
        let name = child_atom_value(text, package_body_start, package_body_end, "name");
        if let (Some(name), Some(version)) = (
            name.as_ref(),
            child_atom_value(text, package_body_start, package_body_end, "version"),
        ) {
            dependencies.push(Dependency {
                name: name.value.as_str().to_owned(),
                requirement: version.value.as_str().to_owned(),
                ecosystem: Opam,
                group: "version".to_owned(),
                hosted_url: None,
                hosted_name: None,
                range: offset_range(text, version.open, version.close + 1),
                requirement_range: offset_range(text, version.value_start, version.value_end),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
            });
        }

        for depends in child_list_stanzas(text, package_body_start, package_body_end, "depends") {
            dependencies.extend(depends_entries(text, depends.body_start, depends.close));
        }
    }

    dependencies
}

#[derive(Clone, Copy)]
struct Stanza {
    open: usize,
    close: usize,
}

type Stanzas = Vec<Stanza>;

struct ListStanza {
    body_start: usize,
    close: usize,
}

struct AtomValue {
    value: String,
    open: usize,
    close: usize,
    value_start: usize,
    value_end: usize,
}

fn package_stanzas(text: &str) -> Stanzas {
    top_level_stanzas(text)
        .into_iter()
        .filter(|stanza| head_symbol(text, stanza.open, stanza.close) == Some("package"))
        .collect()
}

fn top_level_stanzas(text: &str) -> Stanzas {
    let mut stanzas = vec![];
    let mut cursor = 0usize;
    while let Some(relative) = text[cursor..].find('(') {
        let open = cursor + relative;
        let Some(close) = matching_paren(text, open, text.len()) else {
            break;
        };
        stanzas.push(Stanza { open, close });
        cursor = close + 1;
    }
    stanzas
}

fn child_list_stanzas(text: &str, start: usize, end: usize, name: &str) -> Vec<ListStanza> {
    let mut stanzas = vec![];
    let mut cursor = start;
    while cursor < end {
        let Some(open) = next_child_open(text, cursor, end) else {
            break;
        };
        let Some(close) = matching_paren(text, open, end) else {
            break;
        };
        if head_symbol(text, open, close) == Some(name) {
            stanzas.push(ListStanza {
                body_start: symbol_end(text, skip_ascii_whitespace(text, open + 1, close), close),
                close,
            });
        }
        cursor = close + 1;
    }
    stanzas
}

fn child_atom_value(text: &str, start: usize, end: usize, name: &str) -> Option<AtomValue> {
    let mut cursor = start;
    while cursor < end {
        let open = next_child_open(text, cursor, end)?;
        let close = matching_paren(text, open, end)?;
        if head_symbol(text, open, close) == Some(name) {
            let value_start = skip_ascii_whitespace(
                text,
                symbol_end(text, skip_ascii_whitespace(text, open + 1, close), close),
                close,
            );
            let value_end = symbol_end(text, value_start, close);
            let value = text
                .get(value_start..value_end)?
                .trim_matches('"')
                .to_owned();
            return Some(AtomValue {
                value,
                open,
                close,
                value_start,
                value_end,
            });
        }
        cursor = close + 1;
    }
    None
}

fn depends_entries(text: &str, start: usize, end: usize) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut cursor = start;
    while cursor < end {
        cursor = skip_ascii_whitespace(text, cursor, end);
        if cursor >= end {
            break;
        }
        if text.as_bytes().get(cursor) == Some(&b';') {
            cursor = skip_comment(text, cursor, end);
            continue;
        }
        if text.as_bytes().get(cursor) == Some(&b'(') {
            let Some(close) = matching_paren(text, cursor, end) else {
                break;
            };
            if let Some(dependency) = dependency_list_entry(text, cursor, close) {
                dependencies.push(dependency);
            }
            cursor = close + 1;
            continue;
        }
        let name_end = symbol_end(text, cursor, end);
        if let Some(name) = text.get(cursor..name_end).filter(|name| !name.is_empty()) {
            dependencies.push(Dependency {
                name: name.to_owned(),
                requirement: "latest".to_owned(),
                ecosystem: Opam,
                group: "depends".to_owned(),
                hosted_url: None,
                hosted_name: None,
                range: offset_range(text, cursor, name_end),
                requirement_range: offset_range(text, name_end, name_end),
                requirement_prefix: " (>= ".to_owned(),
                requirement_suffix: ")".to_owned(),
            });
        }
        cursor = name_end;
    }
    dependencies
}

fn dependency_list_entry(text: &str, open: usize, close: usize) -> Option<Dependency> {
    let name_start = skip_ascii_whitespace(text, open + 1, close);
    let name_end = symbol_end(text, name_start, close);
    let name = text.get(name_start..name_end)?.trim_matches('"');
    if name.is_empty() {
        return None;
    }
    if let Some(constraint) = first_dune_version_constraint(text, name_end, close) {
        return Some(Dependency {
            name: name.to_owned(),
            requirement: constraint.requirement,
            ecosystem: Opam,
            group: "depends".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, open, close + 1),
            requirement_range: offset_range(text, constraint.range_start, constraint.range_end),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        });
    }
    Some(Dependency {
        name: name.to_owned(),
        requirement: "latest".to_owned(),
        ecosystem: Opam,
        group: "depends".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, open, close + 1),
        requirement_range: offset_range(text, name_end, name_end),
        requirement_prefix: " (>= ".to_owned(),
        requirement_suffix: ")".to_owned(),
    })
}

struct DuneConstraint {
    requirement: String,
    range_start: usize,
    range_end: usize,
}

fn first_dune_version_constraint(text: &str, start: usize, end: usize) -> Option<DuneConstraint> {
    let mut cursor = start;
    while cursor < end {
        let open = text[cursor..end]
            .find('(')
            .map(|relative| cursor + relative)?;
        let close = matching_paren(text, open, end)?;
        let op_start = skip_ascii_whitespace(text, open + 1, close);
        if let Some((operator, after_operator)) = dune_operator(text, op_start, close) {
            let version_start = skip_ascii_whitespace(text, after_operator, close);
            let version_end = symbol_end(text, version_start, close);
            let version = text.get(version_start..version_end)?;
            return Some(DuneConstraint {
                requirement: format!("{operator} {version}"),
                range_start: op_start,
                range_end: version_end,
            });
        }
        cursor = open + 1;
    }
    None
}

fn dune_operator(text: &str, start: usize, end: usize) -> Option<(&str, usize)> {
    for operator in [">=", "<=", "!=", "=", "<", ">"] {
        let operator_end = start + operator.len();
        if operator_end <= end && text.get(start..operator_end) == Some(operator) {
            return Some((operator, operator_end));
        }
    }
    None
}

fn head_symbol<'a>(text: &'a str, open: usize, close: usize) -> Option<&'a str> {
    let start = skip_ascii_whitespace(text, open + 1, close);
    let end = symbol_end(text, start, close);
    text.get(start..end)
}

fn symbol_end(text: &str, mut start: usize, end: usize) -> usize {
    while start < end {
        let byte = text.as_bytes()[start];
        if byte.is_ascii_whitespace() || matches!(byte, b'(' | b')') {
            break;
        }
        start += 1;
    }
    start
}

fn next_child_open(text: &str, mut cursor: usize, end: usize) -> Option<usize> {
    while cursor < end {
        cursor = skip_ascii_whitespace(text, cursor, end);
        match text.as_bytes().get(cursor).copied()? {
            b';' => cursor = skip_comment(text, cursor, end),
            b'(' => return Some(cursor),
            _ => cursor = symbol_end(text, cursor, end),
        }
    }
    None
}

fn skip_comment(text: &str, cursor: usize, end: usize) -> usize {
    text[cursor..end]
        .find('\n')
        .map_or(end, |relative| cursor + relative + 1)
}

fn skip_ascii_whitespace(text: &str, mut offset: usize, end: usize) -> usize {
    while offset < end && text.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
}

fn matching_paren(text: &str, open: usize, limit: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    if bytes.get(open) != Some(&b'(') {
        return None;
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (offset, byte) in bytes.iter().enumerate().take(limit).skip(open) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'(' => depth += 1,
            b')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}
