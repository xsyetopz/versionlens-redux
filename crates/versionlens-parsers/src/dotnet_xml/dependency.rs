use quick_xml::events::BytesStart;

use versionlens_model::Dependency;

use super::{DotnetEventContext, DotnetTagKind, event_name};

mod attrs;
mod package;
mod project;
mod sdk;
mod sdk_tag;

pub(in crate::dotnet_xml) use package::is_package_tag;
use package::package_dependencies_from_tag;
pub(super) use project::project_version_dependency;
use sdk::project_sdk_dependencies;
use sdk_tag::sdk_dependency_from_tag;

pub(super) fn dependencies_from_tag(
    context: &DotnetEventContext<'_>,
    event: &BytesStart<'_>,
    tag_kind: DotnetTagKind,
) -> Vec<Dependency> {
    let Some(tag_name) = event_name(event) else {
        return vec![];
    };
    if tag_name == "Sdk" {
        return sdk_dependency_from_tag(context).into_iter().collect();
    }
    if tag_name == "Project" {
        return project_sdk_dependencies(context);
    }

    if is_package_tag(&tag_name) {
        return package_dependencies_from_tag(context, &tag_name, tag_kind);
    }

    vec![]
}
