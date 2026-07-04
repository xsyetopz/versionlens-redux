use crate::{
    docker::image::split_image_reference,
    model::{Dependency, Ecosystem},
    positions::line_range,
};

pub(crate) fn parse_dockerfile(text: &str) -> Vec<Dependency> {
    text.lines()
        .enumerate()
        .filter_map(|(line_index, line)| parse_from_line(line_index, line))
        .collect()
}

fn parse_from_line(line_index: usize, line: &str) -> Option<Dependency> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || !trimmed.get(..4)?.eq_ignore_ascii_case("FROM") {
        return None;
    }
    if !trimmed
        .as_bytes()
        .get(4)
        .is_some_and(u8::is_ascii_whitespace)
    {
        return None;
    }

    let mut rest = trimmed.get(4..)?.trim_start();
    while rest.starts_with("--") {
        rest = rest.split_once(char::is_whitespace)?.1.trim_start();
    }

    let image_ref = rest.split_whitespace().next()?;
    let image_start = line.find(image_ref)?;
    let image = split_image_reference(image_ref);
    if image.name.is_empty() {
        return None;
    }
    let name_start = image_start + image.name_offset;
    let requirement_start = image_start + image.tag_offset;

    let requirement_prefix = if image.tag.is_empty() { ":" } else { "" };

    Some(Dependency {
        name: image.name.to_owned(),
        requirement: image.tag.to_owned(),
        ecosystem: Ecosystem::Docker,
        group: "FROM".to_owned(),
        hosted_url: (!image.registry.is_empty()).then(|| image.registry.to_owned()),
        hosted_name: None,
        range: line_range(line_index, line, name_start, name_start + image.name.len()),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + image.tag.len(),
        ),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: String::new(),
    })
}
