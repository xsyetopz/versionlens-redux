use crate::{
    model::{Dependency, Ecosystem},
    positions::line_range,
};

const INCOMPATIBLE: &str = "+incompatible";

pub(super) fn parse_go_mod_dependency(
    line_index: usize,
    line: &str,
    group: &str,
    content: &str,
) -> Option<Dependency> {
    let clean = content
        .split_once("//")
        .map_or(content, |(before, _)| before)
        .trim();
    let (name, requirement) = go_mod_dependency_parts(clean)?;
    if has_two_hyphen_separated_version_segments(requirement) {
        return None;
    }

    let name_start = line.find(name)?;
    let requirement_start = if requirement.is_empty() {
        name_start + name.len()
    } else {
        line[name_start + name.len()..].find(requirement)? + name_start + name.len()
    };

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ecosystem::Go,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: line_range(line_index, line, name_start, name_start + name.len()),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + requirement.len(),
        ),
        requirement_prefix: String::new(),
        requirement_suffix: requirement
            .ends_with(INCOMPATIBLE)
            .then(|| INCOMPATIBLE.to_owned())
            .unwrap_or_default(),
    })
}

fn go_mod_dependency_parts(clean: &str) -> Option<(&str, &str)> {
    let mut parts = clean.split_whitespace();
    let name = parts.next()?;
    let requirement = parts.next().unwrap_or_default();

    Some((name, requirement))
}

fn has_two_hyphen_separated_version_segments(version: &str) -> bool {
    version.split('-').count() == 3
}
