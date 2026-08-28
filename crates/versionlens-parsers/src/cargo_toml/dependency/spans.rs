use std::ops::Range as ByteRange;

use crate::positions::dependency_ranges;
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
    let byte_spans = (spans.name, spans.requirement);
    let (range, requirement_range) =
        dependency_ranges(source.text, byte_spans.0, byte_spans.1, true);

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
