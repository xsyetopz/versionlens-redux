use super::trim_end_slash;
use serde_json::Value;
use serde_json::from_str;

const NUGET_PACKAGE_BASE_ADDRESS: &str = "PackageBaseAddress";

/// The package base address for a package, read from a `NuGet` service index.
pub fn dotnet_package_url_from_service_index(body: &str, name: &str) -> Option<String> {
    let name = name.trim();
    let value = from_str::<Value>(body).ok()?;
    let resource = value
        .get("resources")?
        .as_array()?
        .iter()
        .find_map(package_base_address_resource)?;
    Some(dotnet_package_url(resource, &name.to_lowercase()))
}

fn package_base_address_resource(resource: &Value) -> Option<&str> {
    if !resource
        .get("@type")
        .and_then(|value| value.as_str())
        .is_some_and(|kind| kind.contains(NUGET_PACKAGE_BASE_ADDRESS))
    {
        return None;
    }

    resource
        .get("@id")?
        .as_str()
        .map(str::trim)
        .filter(|id| !id.is_empty())
}

pub(super) fn dotnet_default_registry_url(name: &str) -> String {
    dotnet_package_url("https://api.nuget.org/v3-flatcontainer", name)
}

pub(super) fn dotnet_package_url(base_url: &str, name: &str) -> String {
    format!(
        "{}/{}/index.json",
        trim_end_slash(base_url),
        name.to_lowercase()
    )
}
