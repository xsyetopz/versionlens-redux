use marked_yaml::types::MarkedScalarNode;
use std::ops::Range;

use crate::positions::offset_range;
use crate::yaml::{byte_offset, scalar_range};
use versionlens_model::Dependency;

use super::source::PubspecDependencySource;
use versionlens_model::Ecosystem::Pub;

pub(super) fn scalar_dependency_from_source(
    source: &PubspecDependencySource<'_>,
    value: &MarkedScalarNode,
) -> Option<Dependency> {
    let text = source.text;
    let key = source.key;
    let value_range = scalar_range(text, value).unwrap_or_else(|| empty_value_range(text, key));
    let mut requirement_prefix = "".to_owned();
    let mut requirement_suffix = "".to_owned();
    let key_tail = key_line_tail(text, key);
    let requirement = if value.as_str() == "any" {
        "*"
    } else if value.as_str().is_empty()
        && key_tail.is_some_and(|tail| tail.trim_start().starts_with('#'))
    {
        requirement_suffix.push(' ');
        "*"
    } else if value.as_str().is_empty() && key_tail.is_some_and(|tail| tail.trim().is_empty()) {
        requirement_prefix.push(' ');
        value.as_str()
    } else {
        value.as_str()
    };
    let name_start = byte_offset(text, key.span().start()?.character())?;

    Some(Dependency {
        name: key.as_str().to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Pub,
        group: source.group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, name_start, name_start + key.as_str().len()),
        requirement_range: offset_range(text, value_range.start, value_range.end),
        requirement_prefix,
        requirement_suffix,
    })
}

fn key_line_tail<'a>(text: &'a str, key: &MarkedScalarNode) -> Option<&'a str> {
    let key_start = key
        .span()
        .start()
        .and_then(|marker| byte_offset(text, marker.character()))?;
    let line = text[key_start..].lines().next()?;
    let colon = line.find(':')?;
    line.get(colon + 1..)
}

fn empty_value_range(text: &str, key: &MarkedScalarNode) -> Range<usize> {
    let Some(key_start) = key
        .span()
        .start()
        .and_then(|marker| byte_offset(text, marker.character()))
    else {
        return 0..0;
    };
    let line_end = text[key_start..]
        .find('\n')
        .map_or(text.len(), |offset| key_start + offset);
    let Some(colon) = text[key_start..line_end].find(':') else {
        return key_start..key_start;
    };
    let mut cursor = key_start + colon + 1;
    while matches!(text.as_bytes().get(cursor), Some(b' ' | b'\t')) {
        cursor += 1;
    }
    cursor..cursor
}
