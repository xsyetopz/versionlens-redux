use versionlens_parsers::{Dependency, Ecosystem};

use crate::config::DependencyPropertyConfig;

mod group;
mod path;

use group::dependency_property_group;
use path::property_matches;

pub(crate) fn is_enabled<'a>(
    dependency: &Dependency,
    manifest_ecosystem: Ecosystem,
    configs: impl IntoIterator<Item = &'a DependencyPropertyConfig>,
) -> bool {
    let group = dependency_property_group(dependency);
    let member = format!("{group}.{}", dependency.name);
    let mut has_ecosystem_config = false;

    for config in configs
        .into_iter()
        .filter(|config| config.ecosystem == manifest_ecosystem)
    {
        has_ecosystem_config = true;
        if config.properties.iter().any(|property| {
            property_matches(property, &group) || property_matches(property, &member)
        }) {
            return true;
        }
    }

    !has_ecosystem_config
}

#[cfg(test)]
mod tests;
