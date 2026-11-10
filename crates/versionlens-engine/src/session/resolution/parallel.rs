use std::{iter, sync::Arc};

use versionlens_update::{Suggestion, error};

use super::ResolutionRequest;
use super::dependency::ResolveDependencyInput;
use crate::{VersionLensSession, workspace};

const WORKER_PANIC_MESSAGE: &str = "dependency resolution worker panicked";
const OPERATION_TIMEOUT_MESSAGE: &str = "dependency resolution timed out";
impl VersionLensSession {
    pub(crate) async fn resolve_dependencies(
        &self,
        request: ResolutionRequest<'_>,
    ) -> Vec<Suggestion> {
        let session = self;
        let (cache_scope, workspace) =
            session.document_cache_scope_with_graph(request.context, request.input);
        let use_cached_suggestions = request.responses.is_empty() && request.project_bump.is_none();
        let mut dependencies = Vec::with_capacity(request.dependencies.len());
        let mut suggestions = iter::repeat_with(|| None)
            .take(request.dependencies.len())
            .collect::<Vec<_>>();
        for (index, dependency) in request.dependencies.into_iter().enumerate() {
            if use_cached_suggestions
                && let Some(cached) = session.cached_resolved_suggestion(&dependency, &cache_scope)
            {
                suggestions[index] = Some(cached);
            } else {
                dependencies.push((index, dependency));
            }
        }

        let workspace = Arc::new(workspace);
        let workspace_documents = session.workspace_documents();
        let workspace_root = Arc::new(
            workspace::document_path(request.input)
                .and_then(|_| workspace::workspace_root(request.input)),
        );
        let context = Arc::new(request.context.clone());
        let session = Arc::new(session.clone());
        let responses = Arc::new(request.responses.to_vec());
        let document_uri = Arc::new(request.document_uri.to_owned());
        let mut tasks = Vec::with_capacity(dependencies.len());

        for (index, dependency) in dependencies {
            let failure = dependency.clone();
            let workspace = Arc::clone(&workspace);
            let workspace_documents = Arc::clone(&workspace_documents);
            let workspace_root = Arc::clone(&workspace_root);
            let context = Arc::clone(&context);
            let session = Arc::clone(&session);
            let responses = Arc::clone(&responses);
            let document_uri = Arc::clone(&document_uri);
            let operation = request.operation.clone();
            let project_bump = request.project_bump;
            let task = tokio::spawn(async move {
                if operation.is_expired() {
                    return Some(error(dependency, OPERATION_TIMEOUT_MESSAGE.to_owned()));
                }
                session
                    .resolve_dependency_with_responses(ResolveDependencyInput {
                        dependency,
                        workspace: &workspace,
                        workspace_documents: &workspace_documents,
                        workspace_root: workspace_root.as_deref(),
                        document_uri: Some(&document_uri),
                        responses: &responses,
                        project_bump,
                        context: &context,
                        operation: &operation,
                    })
                    .await
            });
            tasks.push((index, task, failure));
        }

        for (index, task, failure) in tasks {
            suggestions[index] = match task.await {
                Ok(suggestion) => suggestion,
                Err(_) => Some(error(failure, WORKER_PANIC_MESSAGE.to_owned())),
            };
        }
        suggestions.into_iter().flatten().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::ProviderSettings;
    use crate::RegistryResponseInput;
    use crate::SessionConfig;
    use versionlens_model::DocumentInput;

    use super::OPERATION_TIMEOUT_MESSAGE;

    use versionlens_model::Ecosystem::Npm;

    fn batched_fixture_input() -> DocumentInput {
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("batched-resolution-preserves-dependency-order.json"),
            None,
        )
    }

    fn batched_fixture_responses() -> Vec<RegistryResponseInput> {
        [
            "dep-00", "dep-01", "dep-02", "dep-03", "dep-04", "dep-05", "dep-06", "dep-07",
            "dep-08", "dep-09", "dep-10", "dep-11",
        ]
        .into_iter()
        .map(|name| {
            RegistryResponseInput::new(
                name.to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
            )
        })
        .collect()
    }

    #[tokio::test]
    async fn batched_resolution_preserves_dependency_order() {
        let session = crate::test_support::test_session(false);
        let responses = batched_fixture_responses();
        let names = responses
            .iter()
            .map(|response| response.package.as_str())
            .collect::<Vec<_>>();

        let output = session
            .resolve_document_with_responses(batched_fixture_input(), &responses)
            .await;
        let resolved_names = output
            .suggestions
            .iter()
            .map(|suggestion| suggestion.dependency.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(resolved_names, names);
    }

    #[tokio::test]
    async fn expired_operation_returns_errors_without_resolving_dependencies() {
        let mut http = versionlens_fetch::standard_http_config();
        http.timeout_ms = 0;
        let session = crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings::default(),
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: false,
            show_suggestion_stats: false,
            show_prereleases: false,
            http,
        });
        let responses = batched_fixture_responses();

        let output = session
            .resolve_document_with_responses(batched_fixture_input(), &responses)
            .await;

        assert_eq!(output.suggestions.len(), responses.len());
        assert!(
            output
                .suggestions
                .iter()
                .all(|suggestion| suggestion.status == "error")
        );
        assert!(
            output
                .suggestions
                .iter()
                .all(|suggestion| suggestion.latest.as_deref() == Some(OPERATION_TIMEOUT_MESSAGE))
        );
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/session/resolution/parallel/tests", name)
    }
}
