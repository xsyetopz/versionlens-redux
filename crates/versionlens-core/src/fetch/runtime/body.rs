use versionlens_model::{Dependency, Ecosystem};
use versionlens_providers::RuntimeSource;

use crate::{
    VersionLensSession, error::FetchError, registry::RegistryContext,
    session::operation::OperationContext,
};

impl VersionLensSession {
    pub(super) async fn fetch_runtime_body(
        &self,
        dependency: &Dependency,
        source: &RuntimeSource,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<String, FetchError> {
        let mut remote = dependency.clone();
        remote.hosted_url = None;
        remote.hosted_name = None;
        remote.canonical_reference = None;
        match source {
            RuntimeSource::GitHubTags { repository, .. } => {
                remote.ecosystem = Ecosystem::GitHub;
                remote.name = (*repository).to_owned();
            }
            RuntimeSource::PackageManager(name) => {
                remote.ecosystem = Ecosystem::Npm;
                remote.name = (*name).to_owned();
            }
            RuntimeSource::BunCanary => remote.ecosystem = Ecosystem::GitHub,
            _ => {}
        }
        let urls = if matches!(
            source,
            RuntimeSource::GitHubTags { .. } | RuntimeSource::PackageManager(_)
        ) {
            self.registry_endpoints_with_context(&remote, context)
                .into_iter()
                .map(|endpoint| endpoint.url)
                .collect()
        } else {
            vec![source.url()]
        };
        let mut failure = None;
        for url in urls {
            if operation.is_expired() {
                return Err(FetchError::OperationTimeout);
            }
            let result = if matches!(source, RuntimeSource::GitHubTags { .. }) {
                self.fetch_github_tags_body(&remote, &url, context, operation)
                    .await
            } else {
                self.get_text_or_status_with_context(&url, remote.ecosystem, context, operation)
                    .await
            };
            match result {
                Ok(Some(body)) => return Ok(body.to_string()),
                Ok(None) => {}
                Err(error) => {
                    failure.get_or_insert(error);
                }
            }
        }
        Err(failure
            .unwrap_or_else(|| super::runtime_error("runtime release source returned no result")))
    }

    pub(super) async fn fetch_runtime_artifact(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<Vec<u8>, FetchError> {
        if operation.is_expired() {
            return Err(FetchError::OperationTimeout);
        }
        let http_config = self.effective_http_config(url, dependency.ecosystem, context);
        let retry_policy = if dependency.ecosystem == Ecosystem::Npm {
            versionlens_http::npm_registry_fetch_retry_policy()
        } else {
            versionlens_http::disabled_retry_policy()
        };
        let response = match operation.remaining_duration() {
            Some(remaining) => {
                versionlens_http::get_bytes_with_accept_and_retry_timeout(
                    url,
                    &http_config,
                    None,
                    retry_policy,
                    remaining,
                )
                .await
            }
            None => {
                versionlens_http::get_bytes_with_accept_and_retry(
                    url,
                    &http_config,
                    None,
                    retry_policy,
                )
                .await
            }
        };
        match response {
            Ok(body) => {
                if operation.is_expired() {
                    Err(FetchError::OperationTimeout)
                } else {
                    Ok(body)
                }
            }
            Err(error) => {
                Err(self.fetch_error_from_http(error, url, "runtime artifact", operation))
            }
        }
    }
}
