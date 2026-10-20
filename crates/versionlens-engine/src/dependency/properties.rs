use versionlens_model::Ecosystem::Dotnet;
use versionlens_model::{Dependency, Ecosystem};

use crate::config::DependencyPropertyConfig;

use versionlens_parsers::path_matches as property_matches;

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

fn dependency_property_group(dependency: &Dependency) -> String {
    match dependency.ecosystem {
        Dotnet => dotnet_dependency_property_group(dependency),
        _ => dependency.group.as_str().to_owned(),
    }
}

fn dotnet_dependency_property_group(dependency: &Dependency) -> String {
    match dependency.group.as_str() {
        "dependencies" => dependency.group.as_str().to_owned(),
        group if group.starts_with("frameworks.") || group.starts_with("runtimes.") => {
            group.to_owned()
        }
        group if group.starts_with("paket.") => group.to_owned(),
        "packages.package" => "packages.package".to_owned(),
        "PropertyGroup" => format!("Project.PropertyGroup.{}", dependency.name),
        "Sdk" | "Project.Sdk" => "Project.Sdk".to_owned(),
        group => format!("Project.ItemGroup.{group}"),
    }
}

#[cfg(test)]
mod tests;
