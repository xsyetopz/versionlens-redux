use crate::positions::offset_range;
use versionlens_model::Dependency;

use super::super::DotnetEventContext;
use super::super::attributes::{attr_value, tag_bounds};
use versionlens_model::Ecosystem::Dotnet;

pub(super) fn project_sdk_dependencies(context: &DotnetEventContext<'_>) -> Vec<Dependency> {
    let (tag_start, tag_end) = tag_bounds(context.text, context.span.start, context.span.end);
    let Some(tag) = context.text.get(tag_start..tag_end) else {
        return vec![];
    };
    let Some(sdk) = attr_value(tag, "Sdk") else {
        return vec![];
    };

    let value_base = tag_start + sdk.range.start;
    let mut dependencies = vec![];
    let mut offset = 0;
    for part in sdk.value.split(';') {
        let trim_start = part.len() - part.trim_start().len();
        let value = part.trim();
        if !value.is_empty() {
            dependencies.push(project_sdk_dependency(
                context.text,
                value_base,
                ProjectSdkPart {
                    value_start: offset + trim_start,
                    value,
                },
            ));
        }
        offset += part.len() + 1;
    }
    dependencies
}

struct ProjectSdkPart<'a> {
    value_start: usize,
    value: &'a str,
}

fn project_sdk_dependency(text: &str, value_base: usize, part: ProjectSdkPart<'_>) -> Dependency {
    let ProjectSdkPart { value_start, value } = part;
    let name_start = value_base + value_start;
    let split_at = value.rfind('/');
    let name_len = split_at.unwrap_or(value.len());
    let requirement = split_at
        .and_then(|index| value.get(index + 1..))
        .filter(|value| !value.is_empty())
        .unwrap_or("*")
        .to_owned();
    let requirement_start = split_at.map_or(name_start + name_len, |index| name_start + index + 1);
    let requirement_end = split_at.map_or(requirement_start, |_| name_start + value.len());

    Dependency {
        name: value[..name_len].to_owned(),
        requirement,
        ecosystem: Dotnet,
        group: "Project.Sdk".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, name_start, name_start + name_len),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: split_at.map_or("/".to_owned(), |_| "".to_owned()),
        requirement_suffix: "".to_owned(),
    }
}
