mod context;
mod files;
mod matches;
mod responses;
mod urls;

use versionlens_providers::RegistryEndpoint;

pub(crate) type RegistryEndpoints = Vec<RegistryEndpoint>;

pub(crate) use context::{RegistryContext, registry_context_from_document_kind_with_files};
pub(crate) use files::RegistryFileSnapshot;
pub(crate) use matches::registry_response_matches;
