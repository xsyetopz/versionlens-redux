use marked_yaml::types::MarkedScalarNode;

use crate::{
    model::{Dependency, Ecosystem},
    positions::offset_range,
    yaml::scalar_range,
};

struct ComposeImageRef<'a> {
    registry: &'a str,
    name: &'a str,
    tag: &'a str,
    name_offset: usize,
    tag_offset: usize,
}

pub(super) fn image_dependency(text: &str, value: &MarkedScalarNode) -> Option<Dependency> {
    if value.as_str().is_empty() {
        return None;
    }

    let value_range = scalar_range(text, value)?;
    let image = split_compose_image_reference(value.as_str());
    if image.name.is_empty() {
        return None;
    }
    let name_start = value_range.start + image.name_offset;
    let requirement_start = value_range.start + image.tag_offset;

    let requirement_prefix = if image.tag.is_empty() { ":" } else { "" };
    let hosted_url = (!image.registry.is_empty()).then(|| image.registry.to_owned());

    Some(Dependency {
        name: image.name.to_owned(),
        requirement: image.tag.to_owned(),
        ecosystem: Ecosystem::Docker,
        group: "services.image".to_owned(),
        hosted_url,
        hosted_name: None,
        range: offset_range(text, name_start, name_start + image.name.len()),
        requirement_range: offset_range(
            text,
            requirement_start,
            requirement_start + image.tag.len(),
        ),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: String::new(),
    })
}

fn split_compose_image_reference(input: &str) -> ComposeImageRef<'_> {
    let registry_len = input.find('/').map_or(0, |slash| slash + 1);
    let registry = registry_len
        .checked_sub(1)
        .and_then(|end| input.get(..end))
        .unwrap_or("");
    let image_with_tag = &input[registry_len..];
    let name_end = image_with_tag.find(':').unwrap_or(image_with_tag.len());
    let tag = image_with_tag
        .get(name_end + 1..)
        .filter(|_| name_end < image_with_tag.len())
        .unwrap_or("");

    ComposeImageRef {
        registry,
        name: &image_with_tag[..name_end],
        tag,
        name_offset: registry_len,
        tag_offset: registry_len + name_end + usize::from(!tag.is_empty()),
    }
}
