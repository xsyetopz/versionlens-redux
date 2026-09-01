use crate::positions::offset_range;
use crate::support;
use crate::yaml::scalar_range;
use marked_yaml::types::MarkedScalarNode;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

pub(super) fn dependency(
    text: &str,
    group: &str,
    key: &MarkedScalarNode,
    value: &MarkedScalarNode,
) -> Option<Dependency> {
    let name_range = scalar_range(text, key)?;
    let value_range = scalar_range(text, value)?;
    let raw_requirement = value.as_str();
    let workspace_reference =
        raw_requirement.starts_with("catalog:") || raw_requirement.starts_with("workspace:");

    let (name, requirement, requirement_start, requirement_end, requirement_prefix) =
        npm_alias_dependency(raw_requirement, value_range.start, value_range.end).unwrap_or((
            key.as_str(),
            raw_requirement,
            value_range.start,
            value_range.end,
            "".to_owned(),
        ));

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Npm,
        group: group.to_owned(),
        hosted_url: workspace_reference.then(|| "workspace".to_owned()),
        hosted_name: None,
        range: offset_range(text, name_range.start, name_range.end),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix,
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    })
}

fn npm_alias_dependency(
    value: &str,
    value_start: usize,
    value_end: usize,
) -> Option<(&str, &str, usize, usize, String)> {
    let spec = value.strip_prefix("npm:")?;
    let Some(at) = spec.rfind('@').filter(|index| *index > 0) else {
        return support::valid_package_alias_name(spec)
            .then(|| (spec, "", value_end, value_end, format!("npm:{spec}@")));
    };
    let name = &spec[..at];
    let requirement = &spec[at + 1..];
    if requirement.is_empty() {
        return None;
    }
    Some((
        name,
        requirement,
        value_start,
        value_end,
        format!("npm:{name}@"),
    ))
}
