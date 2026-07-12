use crate::model::Dependency;
use crate::model::Ecosystem::Python;
use crate::positions::line_range;
use crate::requirements_txt::split::split_requirements_txt_requirement;

pub(super) fn parse_requirement_line(line_index: usize, line: &str) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let offset = line.len() - trimmed.len();
    let (name, mut requirement, mut split) = split_requirements_txt_requirement(trimmed)?;
    let parenthesized = trimmed[..split].trim_end().ends_with('(')
        && trimmed[split + requirement.len()..]
            .trim_start()
            .starts_with(')');
    if !parenthesized && requirements_txt_extras_end_before_requirement(trimmed, name, split) {
        requirement = "";
        split = name.len();
    }
    if let Some(comment_start) = requirement.find('#') {
        requirement = requirement[..comment_start].trim_end();
    }
    let requirement_start = offset + split;
    let requirement_end = requirement_start + requirement.len();

    let requirement_prefix = if requirement.is_empty() { "==" } else { "" };
    let requirement_value = requirements_txt_descriptor_version(requirement);

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement_value,
        ecosystem: Python,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(line_index, line, offset, offset + name.len()),
        requirement_range: line_range(line_index, line, requirement_start, requirement_end),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: if parenthesized { ")" } else { "" }.to_owned(),
    })
}

fn requirements_txt_extras_end_before_requirement(raw: &str, name: &str, split: usize) -> bool {
    let Some(extra_start) = raw.find('[') else {
        return false;
    };
    extra_start == name.len() && raw.find(']').is_some_and(|extra_end| extra_end < split)
}

fn requirements_txt_descriptor_version(raw: &str) -> String {
    for operator in ["===", "==", "~=", ">=", "<=", "!=", ">", "<"] {
        let Some(after_operator) = raw.strip_prefix(operator) else {
            continue;
        };
        let version = after_operator.trim_start_matches(crate::is_whitespace);
        if version.len() != after_operator.len() {
            return format!("{operator}{version}");
        }
        return raw.to_owned();
    }
    raw.to_owned()
}
