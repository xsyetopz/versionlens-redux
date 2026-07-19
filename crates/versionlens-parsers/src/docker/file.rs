use crate::docker::image::split_image_reference;
use crate::positions::line_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Docker;

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
        .is_some_and(|value| value.is_ascii_whitespace())
    {
        return None;
    }

    let mut rest = trimmed.get(4..)?.trim_start();
    while rest.starts_with("--") {
        rest = rest
            .split_once(|value: char| value.is_whitespace())?
            .1
            .trim_start();
    }

    let image_ref = rest.split_whitespace().next()?;
    let image_start = line.find(image_ref)?;
    let image = split_image_reference(image_ref);
    if image.name.is_empty() {
        return None;
    }
    let name_start = image_start + image.name_offset;
    let (requirement, requirement_start, requirement_prefix) =
        if image.tag.is_empty() && !image.digest.is_empty() {
            (image.digest, image_start + image.digest_offset, "@")
        } else {
            (
                image.tag,
                image_start + image.tag_offset,
                if image.tag.is_empty() { ":" } else { "" },
            )
        };

    Some(Dependency {
        name: image.name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Docker,
        group: "FROM".to_owned(),
        hosted_url: (!image.registry.is_empty()).then(|| image.registry.to_owned()),
        hosted_name: None,
        range: line_range(line_index, line, name_start, name_start + image.name.len()),
        requirement_range: line_range(
            line_index,
            line,
            requirement_start,
            requirement_start + requirement.len(),
        ),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: "".to_owned(),
    })
}
