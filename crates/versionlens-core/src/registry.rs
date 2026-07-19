mod context;
mod matches;
mod responses;
mod urls;

use versionlens_providers::RegistryEndpoint;

pub(crate) type RegistryEndpoints = Vec<RegistryEndpoint>;

#[cfg(test)]
pub(crate) use context::registry_context_from_document;
pub(crate) use context::{RegistryContext, registry_context_from_document_kind};
pub(crate) use matches::registry_response_matches;
