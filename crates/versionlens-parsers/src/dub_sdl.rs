use crate::model::Dependency;
use crate::model::Ecosystem::Dub;
use crate::positions::line_range;

pub(crate) fn parse_dub_sdl(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_dub_sdl_line(line_index, line))
        .collect()
}

fn parse_dub_sdl_line(line_index: usize, line: &str) -> Option<Dependency> {
    let content = line.split_once('#').map_or(line, |(before, _)| before);
    let trimmed = content.trim_start();
    let content_start = content.len() - trimmed.len();
    let rest = trimmed.strip_prefix("dependency")?;
    if !rest.starts_with(|value: char| value.is_whitespace()) {
        return None;
    }
    let name_start = content_start + line[content_start..].find('"')? + 1;
    let name = quoted_value_at(line, name_start)?;
    let attributes = &line[name_start + name.len() + 1..];
    let (requirement, requirement_start, requirement_field) =
        dependency_requirement(line, attributes)?;

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Dub,
        group: "dependencies".to_owned(),
        hosted_url: (requirement_field == "path").then_some("path".to_owned()),
        hosted_name: None,
        range: line_range(line_index, line, name_start, name_start + name.len()),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + requirement.len(),
        ),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn dependency_requirement<'a>(
    line: &'a str,
    attributes: &'a str,
) -> Option<(&'a str, usize, &'static str)> {
    if let Some((path, start)) = attribute_value(line, attributes, "path") {
        return Some((path, start, "path"));
    }
    if let Some((repository, start)) = attribute_value(line, attributes, "repository") {
        return Some((repository, start, "repository"));
    }
    attribute_value(line, attributes, "version").map(|(version, start)| (version, start, "version"))
}

fn attribute_value<'a>(
    line: &'a str,
    attributes: &'a str,
    attribute: &str,
) -> Option<(&'a str, usize)> {
    let pattern = format!("{attribute}=\"");
    let start = attributes.find(&pattern)? + pattern.len();
    let value_start = line.len() - attributes.len() + start;
    let value = quoted_value_at(line, value_start)?;
    Some((value, value_start))
}

fn quoted_value_at(line: &str, value_start: usize) -> Option<&str> {
    let value_end = line[value_start..].find('"')? + value_start;
    Some(&line[value_start..value_end])
}
