use versionlens_vscode_model::{Position, Range};

use crate::model::{Dependency, Ecosystem};
use crate::positions::offset_range;

pub(super) struct CargoDependencySource<'a> {
    pub(super) text: &'a str,
    pub(super) group: &'a str,
    pub(super) name: &'a str,
    pub(super) requirement: &'a str,
    pub(super) hosted_url: Option<&'a str>,
}

pub(super) struct CargoDependencySpans {
    pub(super) name: Option<std::ops::Range<usize>>,
    pub(super) requirement: Option<std::ops::Range<usize>>,
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
        ecosystem: Ecosystem::Cargo,
        group: source.group.to_owned(),
        hosted_url: source.hosted_url.map(str::to_owned),
        hosted_name: None,
        range,
        requirement_range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
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
