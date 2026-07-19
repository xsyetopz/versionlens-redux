use std::ops::Range as ByteRange;

use versionlens_model::{Position, Range};

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Cargo;

pub(super) struct CargoDependencySource<'a> {
    pub(super) text: &'a str,
    pub(super) group: &'a str,
    pub(super) name: &'a str,
    pub(super) hosted_name: Option<&'a str>,
    pub(super) requirement: &'a str,
    pub(super) hosted_url: Option<&'a str>,
}

pub(super) struct CargoDependencySpans {
    pub(super) name: Option<ByteRange<usize>>,
    pub(super) requirement: Option<ByteRange<usize>>,
}

pub(super) fn cargo_dependency_from_span(
    source: CargoDependencySource<'_>,
    spans: CargoDependencySpans,
) -> Dependency {
    let range = spans
        .name
        .map(|span| offset_range(source.text, span.start, span.end))
        .unwrap_or_else(empty_range);
    let requirement_range = spans
        .requirement
        .map(|span| string_content_range(source.text, span.start, span.end))
        .unwrap_or(range);

    Dependency {
        name: source.name.to_owned(),
        requirement: source.requirement.to_owned(),
        ecosystem: Cargo,
        group: source.group.to_owned(),
        hosted_url: source.hosted_url.map(|value| value.to_owned()),
        hosted_name: source.hosted_name.map(|value| value.to_owned()),
        range,
        requirement_range,
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn string_content_range(text: &str, start: usize, end: usize) -> Range {
    let content_start = start + usize::from(text.as_bytes().get(start) == Some(&b'"'));
    let content_end = end.saturating_sub(usize::from(
        end > start && text.as_bytes().get(end - 1) == Some(&b'"'),
    ));
    offset_range(text, content_start, content_end)
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
