use crate::docker::image::split_image_reference;
use crate::support::{SourcedDependency, sourced_dependency};
use crate::yaml::scalar_content_offset;
use versionlens_model::{Dependency, Ecosystem};

pub(super) fn dependency(
    text: &str,
    value: &str,
    span: std::ops::Range<usize>,
) -> Option<Dependency> {
    let image = split_image_reference(value);
    if image.name.is_empty() {
        return None;
    }
    let (requirement, offset, prefix) = if image.digest.is_empty() {
        (
            image.tag,
            image.tag_offset,
            if image.tag.is_empty() { ":" } else { "" },
        )
    } else {
        (image.digest, image.digest_offset, "")
    };
    let source_offset =
        |offset| scalar_content_offset(text, span.clone(), "docker://".len() + offset);
    let mut dependency = sourced_dependency(SourcedDependency {
        text,
        ecosystem: Ecosystem::Docker,
        group: "uses",
        name: image.name,
        requirement,
        hosted_url: (!image.registry.is_empty()).then_some(image.registry),
        hosted_name: None,
        range: source_offset(image.name_offset)?
            ..source_offset(image.name_offset + image.name.len())?,
        requirement_range: source_offset(offset)?..source_offset(offset + requirement.len())?,
    });
    dependency.requirement_prefix = prefix.to_owned();
    Some(dependency)
}
