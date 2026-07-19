use crate::docker::image::split_image_reference;
use crate::positions::offset_range;
use crate::yaml::scalar_range;
use marked_yaml::types::MarkedScalarNode;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Docker;

pub(super) fn image_dependency(text: &str, value: &MarkedScalarNode) -> Option<Dependency> {
    if value.as_str().is_empty() {
        return None;
    }

    let value_range = scalar_range(text, value)?;
    let image = split_image_reference(value.as_str());
    if image.name.is_empty() {
        return None;
    }
    let name_start = value_range.start + image.name_offset;
    let (requirement, requirement_start, requirement_prefix) =
        if image.tag.is_empty() && !image.digest.is_empty() {
            (image.digest, value_range.start + image.digest_offset, "@")
        } else {
            (
                image.tag,
                value_range.start + image.tag_offset,
                if image.tag.is_empty() { ":" } else { "" },
            )
        };

    let hosted_url = (!image.registry.is_empty()).then(|| image.registry.to_owned());

    Some(Dependency {
        name: image.name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Docker,
        group: "services.image".to_owned(),
        hosted_url,
        hosted_name: None,
        range: offset_range(text, name_start, name_start + image.name.len()),
        requirement_range: offset_range(
            text,
            requirement_start,
            requirement_start + requirement.len(),
        ),
        requirement_prefix: requirement_prefix.to_owned(),
        requirement_suffix: "".to_owned(),
    })
}
