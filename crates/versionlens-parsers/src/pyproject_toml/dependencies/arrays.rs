use toml_edit::Value as TomlValue;

use crate::model::{Dependency, Ecosystem};
use crate::positions::offset_range;
use crate::requirements_txt::split_python_requirement;

use super::spans::string_content_bounds;

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
        let Some((name, requirement, split)) = split_python_requirement(raw) else {
            continue;
        };
        let Some(span) = item.span() else {
            continue;
        };
        let content = string_content_bounds(text, span.start, span.end);
        out.push(Dependency {
            name: name.to_owned(),
            requirement: requirement.to_owned(),
            ecosystem: Ecosystem::Python,
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
                String::new()
            },
            requirement_suffix: String::new(),
        });
    }
}
