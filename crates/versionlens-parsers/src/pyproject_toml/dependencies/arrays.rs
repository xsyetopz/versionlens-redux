use toml_edit::Value as TomlValue;

use crate::positions::offset_range;
use crate::requirements_txt::split_python_requirement;
use versionlens_model::Dependency;

use super::spans::string_content_bounds;
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
        });
    }
}
