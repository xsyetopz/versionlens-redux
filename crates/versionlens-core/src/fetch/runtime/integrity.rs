use versionlens_model::Dependency;
use versionlens_providers::RuntimeSource;

use crate::{VersionLensSession, error::FetchError};
use crate::{registry::RegistryContext, session::operation::OperationContext};

pub(super) struct RuntimeReplacementRequest<'a> {
    pub(super) dependency: &'a Dependency,
    pub(super) source: &'a RuntimeSource,
    pub(super) body: &'a str,
    pub(super) selected: &'a str,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}

impl VersionLensSession {
    pub(super) fn runtime_replacement(
        &self,
        request: RuntimeReplacementRequest<'_>,
    ) -> Result<Option<String>, FetchError> {
        let Some((current_url, selected_url)) = request
            .source
            .integrity_artifact_urls(
                request.body,
                &request.dependency.requirement,
                request.selected,
            )
            .map_err(super::runtime_error)?
        else {
            return request
                .source
                .verified_replacement(
                    request.body,
                    &request.dependency.requirement,
                    request.selected,
                )
                .map_err(super::runtime_error);
        };
        let current_artifact = self.fetch_runtime_artifact(
            request.dependency,
            &current_url,
            request.context,
            request.operation,
        )?;
        request
            .source
            .verify_artifact_integrity(&request.dependency.requirement, &current_artifact)
            .map_err(super::runtime_error)?;
        let selected_artifact = if current_url == selected_url {
            current_artifact
        } else {
            self.fetch_runtime_artifact(
                request.dependency,
                &selected_url,
                request.context,
                request.operation,
            )?
        };
        request
            .source
            .artifact_integrity_replacement(
                &request.dependency.requirement,
                request.selected,
                &selected_artifact,
            )
            .map(Some)
            .map_err(super::runtime_error)
    }
}
