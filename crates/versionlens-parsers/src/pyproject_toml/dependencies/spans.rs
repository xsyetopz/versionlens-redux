use std::ops::Range as ByteRange;

use versionlens_model::Dependency;

use crate::positions::dependency_ranges;
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
    let (range, requirement_range) =
        dependency_ranges(source.text, spans.name, spans.requirement, true);

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
        canonical_reference: None,
    }
}
