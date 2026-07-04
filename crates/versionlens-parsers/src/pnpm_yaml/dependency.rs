use marked_yaml::types::MarkedScalarNode;

use crate::{
    model::{Dependency, Ecosystem},
    positions::offset_range,
    yaml::scalar_range,
};

pub(super) fn dependency(
    text: &str,
    group: &str,
    key: &MarkedScalarNode,
    value: &MarkedScalarNode,
) -> Option<Dependency> {
    let name_range = scalar_range(text, key)?;
    let value_range = scalar_range(text, value)?;

    Some(Dependency {
        name: key.as_str().to_owned(),
        requirement: value.as_str().to_owned(),
        ecosystem: Ecosystem::Npm,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, name_range.start, name_range.end),
        requirement_range: offset_range(text, value_range.start, value_range.end),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    })
}
