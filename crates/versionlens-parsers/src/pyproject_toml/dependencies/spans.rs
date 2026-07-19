use std::ops::Range as ByteRange;

use versionlens_model::{Position, Range};

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Python;

pub(in crate::pyproject_toml) struct PythonDependencySource<'a> {
    pub(in crate::pyproject_toml) text: &'a str,
    pub(in crate::pyproject_toml) group: &'a str,
    pub(in crate::pyproject_toml) name: &'a str,
    pub(in crate::pyproject_toml) requirement: &'a str,
    pub(in crate::pyproject_toml) hosted_url: Option<&'a str>,
}

pub(in crate::pyproject_toml) struct PythonDependencySpans {
    pub(in crate::pyproject_toml) name: Option<ByteRange<usize>>,
    pub(in crate::pyproject_toml) requirement: Option<ByteRange<usize>>,
}

pub(in crate::pyproject_toml) fn dependency_from_span(
    source: PythonDependencySource<'_>,
    spans: PythonDependencySpans,
) -> Dependency {
    let range = spans
        .name
        .map(|span| offset_range(source.text, span.start, span.end))
        .unwrap_or_else(empty_range);
    let requirement_range = spans
        .requirement
        .map(|span| {
            let content = string_content_bounds(source.text, span.start, span.end);
            offset_range(source.text, content.start, content.end)
        })
        .unwrap_or(range);

    Dependency {
        name: source.name.to_owned(),
        requirement: source.requirement.to_owned(),
        ecosystem: Python,
        group: source.group.to_owned(),
        hosted_url: source.hosted_url.map(|value| value.to_owned()),
        hosted_name: None,
        range,
        requirement_range,
        requirement_prefix: if source.requirement.is_empty() {
            "==".to_owned()
        } else {
            "".to_owned()
        },
        requirement_suffix: "".to_owned(),
    }
}

pub(super) fn string_content_bounds(text: &str, start: usize, end: usize) -> ByteRange<usize> {
    let content_start = start + usize::from(text.as_bytes().get(start) == Some(&b'"'));
    let content_end = end.saturating_sub(usize::from(
        end > start && text.as_bytes().get(end - 1) == Some(&b'"'),
    ));
    content_start..content_end
}

fn empty_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}
