use versionlens_model::Dependency;

use crate::dotnet_xml::DotnetEventContext;
use crate::dotnet_xml::dependency::attrs::{DotnetMissingVersionAttrs, missing_version_dependency};

const MISSING_VERSION_ATTRS: &[&str] = &["Include", "Update"];

pub(super) fn missing_version_package_dependency(
    context: &DotnetEventContext<'_>,
    group: &str,
) -> Option<Dependency> {
    MISSING_VERSION_ATTRS.iter().find_map(|name_attr| {
        missing_version_dependency(context, DotnetMissingVersionAttrs { group, name_attr })
    })
}
