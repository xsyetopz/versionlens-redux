use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Dotnet;

pub(super) fn dependency_property_group(dependency: &Dependency) -> String {
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
