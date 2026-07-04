use crate::model::{Dependency, Ecosystem};
use crate::positions::offset_range;

use super::super::OpenProjectVersion;

pub(in crate::dotnet_xml) fn project_version_dependency(
    text: &str,
    name: String,
    open: &OpenProjectVersion,
) -> Dependency {
    let value = open.value.trim();
    let value_start = text[open.text_start..]
        .find(value)
        .map_or(open.text_start, |offset| open.text_start + offset);

    Dependency {
        name,
        requirement: value.to_owned(),
        ecosystem: Ecosystem::Dotnet,
        group: "PropertyGroup".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, value_start, value_start + value.len()),
        requirement_range: offset_range(text, value_start, value_start + value.len()),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}
