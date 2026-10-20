mod context;
mod files;
mod responses;
mod urls;

use crate::RegistryResponseInput;
use versionlens_model::Dependency;
use versionlens_providers::RegistryEndpoint;

pub(crate) type RegistryEndpoints = Vec<RegistryEndpoint>;

pub(crate) use context::{RegistryContext, registry_context_from_document_kind_with_files};
pub(crate) use files::RegistryFileSnapshot;

pub(crate) fn registry_response_matches(
    response: &RegistryResponseInput,
    dependency: &Dependency,
) -> bool {
    response.ecosystem == dependency.ecosystem
        && (response.package == dependency.name
            || dependency
                .hosted_name
                .as_deref()
                .is_some_and(|hosted_name| response.package == hosted_name))
}
