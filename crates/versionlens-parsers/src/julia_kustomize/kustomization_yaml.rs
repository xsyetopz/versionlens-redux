use crate::docker::image::split_image_reference;
use crate::positions::offset_range;
use crate::support;
use crate::yaml::scalar_range;
use marked_yaml::types::Node::{
    Mapping as YamlMapping, Scalar as YamlScalar, Sequence as YamlSequence,
};
use marked_yaml::types::{MarkedMappingNode, MarkedScalarNode};
use versionlens_model::Ecosystem::Docker;

use versionlens_model::Dependency;

type KustomizationImageDependency = Option<Dependency>;

struct TaggedImageDependency<'a> {
    original_name: &'a MarkedScalarNode,
    registry: &'a str,
    name: &'a str,
    name_start: usize,
    new_tag: &'a MarkedScalarNode,
}

struct ImageDependencyParts<'a> {
    name: &'a str,
    requirement: &'a str,
    registry: &'a str,
    hosted_name: Option<String>,
    name_start: usize,
    requirement_start: usize,
    requirement_prefix: &'a str,
}

pub(crate) fn parse_kustomization_yaml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    if !dependency_paths.is_empty() && !dependency_paths.contains(&"images") {
        return vec![];
    }

    support::with_yaml_mapping(text, |root| {
        let Some(YamlSequence(images)) = root.get_node("images") else {
            return vec![];
        };
        images
            .iter()
            .filter_map(|entry| match entry {
                YamlMapping(image) => image_dependency(text, image),
                YamlScalar(image) => scalar_image_dependency(text, image),
                YamlSequence(_) => None,
            })
            .collect()
    })
    .unwrap_or_default()
}

fn image_dependency(text: &str, image: &MarkedMappingNode) -> KustomizationImageDependency {
    let original_name = image.get_scalar("name")?;
    let new_name = image.get_scalar("newName");
    let identity = new_name.unwrap_or(original_name);
    let identity_range = scalar_range(text, identity)?;
    let image_ref = split_image_reference(identity.as_str());
    if image_ref.name.is_empty() {
        return None;
    }

    let name_start = identity_range.start + image_ref.name_offset;
    if let Some(digest) = image.get_scalar("digest") {
        return digest_dependency(text, original_name, image_ref.name, name_start, digest);
    }

    if let Some(new_tag) = image.get_scalar("newTag") {
        return tagged_dependency(
            text,
            TaggedImageDependency {
                original_name,
                registry: image_ref.registry,
                name: image_ref.name,
                name_start,
                new_tag,
            },
        );
    }

    let (requirement, requirement_start, requirement_prefix) = if image_ref.tag.is_empty() {
        ("", identity_range.start + image_ref.tag_offset, ":")
    } else {
        (
            image_ref.tag,
            identity_range.start + image_ref.tag_offset,
            "",
        )
    };

    Some(build_image_dependency(
        text,
        ImageDependencyParts {
            name: image_ref.name,
            requirement,
            registry: image_ref.registry,
            hosted_name: new_name.map(|_| original_name.as_str().to_owned()),
            name_start,
            requirement_start,
            requirement_prefix,
        },
    ))
}

fn scalar_image_dependency(text: &str, image: &MarkedScalarNode) -> KustomizationImageDependency {
    let value_range = scalar_range(text, image)?;
    let image_ref = split_image_reference(image.as_str());
    if image_ref.name.is_empty() {
        return None;
    }
    let name_start = value_range.start + image_ref.name_offset;
    let (requirement, requirement_start, requirement_prefix) = if image_ref.tag.is_empty() {
        ("", value_range.start + image_ref.tag_offset, ":")
    } else {
        (image_ref.tag, value_range.start + image_ref.tag_offset, "")
    };

    Some(build_image_dependency(
        text,
        ImageDependencyParts {
            name: image_ref.name,
            requirement,
            registry: image_ref.registry,
            hosted_name: None,
            name_start,
            requirement_start,
            requirement_prefix,
        },
    ))
}

fn build_image_dependency(text: &str, parts: ImageDependencyParts<'_>) -> Dependency {
    Dependency {
        name: parts.name.to_owned(),
        requirement: parts.requirement.to_owned(),
        ecosystem: Docker,
        group: "images".to_owned(),
        hosted_url: (!parts.registry.is_empty()).then(|| parts.registry.to_owned()),
        hosted_name: parts.hosted_name,
        range: offset_range(text, parts.name_start, parts.name_start + parts.name.len()),
        requirement_range: offset_range(
            text,
            parts.requirement_start,
            parts.requirement_start + parts.requirement.len(),
        ),
        requirement_prefix: parts.requirement_prefix.to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn tagged_dependency(text: &str, image: TaggedImageDependency<'_>) -> KustomizationImageDependency {
    let tag_range = scalar_range(text, image.new_tag)?;
    Some(Dependency {
        name: image.name.to_owned(),
        requirement: image.new_tag.as_str().to_owned(),
        ecosystem: Docker,
        group: "images".to_owned(),
        hosted_url: (!image.registry.is_empty()).then(|| image.registry.to_owned()),
        hosted_name: Some(image.original_name.as_str().to_owned())
            .filter(|original| original != image.name),
        range: offset_range(text, image.name_start, image.name_start + image.name.len()),
        requirement_range: offset_range(text, tag_range.start, tag_range.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn digest_dependency(
    text: &str,
    original_name: &MarkedScalarNode,
    name: &str,
    name_start: usize,
    digest: &MarkedScalarNode,
) -> KustomizationImageDependency {
    let digest_range = scalar_range(text, digest)?;
    Some(Dependency {
        name: name.to_owned(),
        requirement: digest.as_str().to_owned(),
        ecosystem: Docker,
        group: "images".to_owned(),
        hosted_url: None,
        hosted_name: Some(original_name.as_str().to_owned()).filter(|original| original != name),
        range: offset_range(text, name_start, name_start + name.len()),
        requirement_range: offset_range(text, digest_range.start, digest_range.end),
        requirement_prefix: "digest: ".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}
