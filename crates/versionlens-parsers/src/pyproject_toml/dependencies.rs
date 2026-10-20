use crate::positions::dependency_ranges;
use crate::positions::{offset_range, string_content_bounds};
use crate::requirements_txt::split_python_requirement;
use std::ops::Range as ByteRange;
use toml_edit::{Key, Value as TomlValue};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Python;

pub(in crate::pyproject_toml) fn collect_requirement_array(
    text: &str,
    group: &str,
    value: &TomlValue,
    out: &mut Vec<Dependency>,
) {
    let Some(array) = value.as_array() else {
        return;
    };

    for item in array.iter() {
        let Some(raw) = item.as_str() else {
            continue;
        };
        let Some((name, requirement, split, requirement_suffix)) = split_python_requirement(raw)
        else {
            continue;
        };
        let Some(span) = item.span() else {
            continue;
        };
        let content = string_content_bounds(text, span.start, span.end);
        out.push(Dependency {
            name: name.to_owned(),
            requirement: requirement.to_owned(),
            ecosystem: Python,
            group: group.to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: offset_range(text, content.start, content.start + name.len()),
            requirement_range: offset_range(
                text,
                content.start + split,
                content.start + split + requirement.len(),
            ),
            requirement_prefix: if requirement.is_empty() {
                "==".to_owned()
            } else {
                "".to_owned()
            },
            requirement_suffix: requirement_suffix.to_owned(),
            canonical_reference: None,
        });
    }
}

pub(in crate::pyproject_toml) struct PythonKeyedDependencyInput<'a> {
    pub(in crate::pyproject_toml) text: &'a str,
    pub(in crate::pyproject_toml) group: &'a str,
    pub(in crate::pyproject_toml) name: &'a str,
    pub(in crate::pyproject_toml) value: &'a TomlValue,
    pub(in crate::pyproject_toml) key: &'a Key,
}

pub(in crate::pyproject_toml) fn keyed_dependency(
    input: PythonKeyedDependencyInput<'_>,
) -> Option<Dependency> {
    if let Some(requirement) = input.value.as_str() {
        return Some(dependency_from_span(
            PythonDependencySource {
                text: input.text,
                group: input.group,
                name: input.name,
                requirement,
                hosted_url: None,
            },
            PythonDependencySpans {
                name: input.key.span(),
                requirement: input.value.span(),
            },
        ));
    }

    let requirement = inline_dependency_requirement(input.value)?;
    Some(dependency_from_span(
        PythonDependencySource {
            text: input.text,
            group: input.group,
            name: input.name,
            requirement: requirement.as_str()?,
            hosted_url: inline_dependency_source(input.value),
        },
        PythonDependencySpans {
            name: input.key.span(),
            requirement: requirement.span(),
        },
    ))
}

fn inline_dependency_requirement(value: &TomlValue) -> Option<&TomlValue> {
    let inline = value.as_inline_table()?;
    ["version", "path", "git"]
        .into_iter()
        .find_map(|field| inline.get(field))
        .filter(|value| value.as_str().is_some())
}

fn inline_dependency_source(value: &TomlValue) -> Option<&str> {
    value
        .as_inline_table()?
        .get("source")
        .and_then(|value| value.as_str())
}

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
