use crate::response::versions::object_version_values;
use serde_json::Value;

pub(crate) fn latest_vcpkg_version(
    value: &Value,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = object_version_values(value)?;

    versionlens_versions::latest_version_with_prerelease_tags(
        versions,
        include_prereleases,
        prerelease_tags,
    )
}
