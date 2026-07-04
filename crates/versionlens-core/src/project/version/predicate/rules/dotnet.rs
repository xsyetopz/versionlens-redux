use versionlens_parsers::Dependency;

pub(super) fn is_dotnet_property_version(dependency: &Dependency) -> bool {
    dependency.group == "PropertyGroup"
        && matches!(dependency.name.as_str(), "Version" | "AssemblyVersion")
}
