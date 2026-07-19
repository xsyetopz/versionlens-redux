use crate::positions::line_range;
use std::borrow::Cow;
use std::borrow::Cow::{Borrowed as CowBorrowed, Owned as CowOwned};

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Go;

const INCOMPATIBLE: &str = "+incompatible";

pub(super) fn parse_go_mod_dependency(
    line_index: usize,
    line: &str,
    group: &str,
    content: &str,
) -> Option<Dependency> {
    let clean = content
        .split_once("//")
        .map_or(content, |(before, _)| before)
        .trim();
    let parts = go_mod_dependency_parts(group, clean)?;

    let name_raw_start = line.find(parts.name.raw)?;
    let name_start = name_raw_start + parts.name.value_start;
    let name_value = parts.name.value.as_ref();
    let requirement_value = parts.requirement.value.as_ref();

    let requirement_start = if requirement_value.is_empty() {
        name_start + parts.name.value_len
    } else if group == "use" {
        name_start
    } else {
        line[name_raw_start + parts.name.raw.len()..].find(parts.requirement.raw)?
            + name_raw_start
            + parts.name.raw.len()
            + parts.requirement.value_start
    };

    Some(Dependency {
        name: name_value.to_owned(),
        requirement: requirement_value.to_owned(),
        ecosystem: Go,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(
            line_index,
            line,
            name_start,
            name_start + parts.name.value_len,
        ),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + parts.requirement.value_len,
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: requirement_value
            .ends_with(INCOMPATIBLE)
            .then(|| INCOMPATIBLE.to_owned())
            .unwrap_or_default(),
    })
}

struct GoModToken<'a> {
    raw: &'a str,
    value: Cow<'a, str>,
    value_start: usize,
    value_len: usize,
}

struct GoModDependencyParts<'a> {
    name: GoModToken<'a>,
    requirement: GoModToken<'a>,
}

fn go_mod_token(raw: &str) -> GoModToken<'_> {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        let inner = &raw[1..raw.len() - 1];
        GoModToken {
            raw,
            value: unescape_go_interpreted_string(inner),
            value_start: 1,
            value_len: inner.len(),
        }
    } else if raw.len() >= 2 && raw.starts_with('`') && raw.ends_with('`') {
        let inner = &raw[1..raw.len() - 1];
        GoModToken {
            raw,
            value: CowBorrowed(inner),
            value_start: 1,
            value_len: inner.len(),
        }
    } else {
        GoModToken {
            raw,
            value: CowBorrowed(raw),
            value_start: 0,
            value_len: raw.len(),
        }
    }
}

fn unescape_go_interpreted_string(value: &str) -> Cow<'_, str> {
    if !value.contains('\\') {
        return CowBorrowed(value);
    }

    let mut unescaped: String = crate::default();
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(escaped) = chars.next() {
                unescaped.push(escaped);
            } else {
                unescaped.push(ch);
            }
        } else {
            unescaped.push(ch);
        }
    }

    CowOwned(unescaped)
}

fn empty_go_mod_token() -> GoModToken<'static> {
    GoModToken {
        raw: "",
        value: CowBorrowed(""),
        value_start: 0,
        value_len: 0,
    }
}

fn go_mod_dependency_parts<'a>(group: &str, clean: &'a str) -> Option<GoModDependencyParts<'a>> {
    if group == "replace" {
        return go_mod_replace_dependency_parts(clean);
    }
    if group == "use" {
        let path = clean.split_whitespace().next()?;
        return Some(GoModDependencyParts {
            name: go_mod_token(path),
            requirement: go_mod_token(path),
        });
    }

    let mut parts = clean.split_whitespace();
    let name = go_mod_token(parts.next()?);
    let requirement = match parts.next() {
        Some(requirement) => go_mod_token(requirement),
        None => empty_go_mod_token(),
    };

    Some(GoModDependencyParts { name, requirement })
}

fn go_mod_replace_dependency_parts(clean: &str) -> Option<GoModDependencyParts<'_>> {
    let (left, right) = clean.split_once("=>")?;
    let mut left_parts = left.split_whitespace();
    let original_name = go_mod_token(left_parts.next()?);

    let mut right_parts = right.split_whitespace();
    let replacement_name = go_mod_token(right_parts.next()?);
    if is_local_go_replacement(replacement_name.value.as_ref()) {
        return Some(GoModDependencyParts {
            name: original_name,
            requirement: replacement_name,
        });
    }

    Some(GoModDependencyParts {
        name: replacement_name,
        requirement: match right_parts.next() {
            Some(requirement) => go_mod_token(requirement),
            None => empty_go_mod_token(),
        },
    })
}

fn is_local_go_replacement(replacement: &str) -> bool {
    replacement.starts_with('/')
        || replacement.starts_with("./")
        || replacement.starts_with("../")
        || replacement.starts_with("~/")
}
