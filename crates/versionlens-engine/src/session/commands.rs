use std::collections::HashSet;
use std::panic::resume_unwind;
use std::sync::Arc;
use std::time::Instant;

use versionlens_model::{Dependency, DocumentInput, ManifestKind};
use versionlens_update::Suggestion;
use versionlens_update::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    NotSupported as StatusNotSupported, RateLimited as StatusRateLimited,
    Unresolved as StatusUnresolved, UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_update::bulk_update_edits;
use versionlens_update::sort_dependency_edits;
use versionlens_update::update_edits;

use super::documents::DependencySuggestionsRequest;
use super::operation::OperationContext;
use crate::VersionLensSession;
use crate::cache::vulnerability_cache_key;
use crate::command::{filter_update_command, project_version_bump};
use crate::contract::{RegistryResponseInput, ResolveDocumentOutput};
use crate::presentation::status::to_u32;
use crate::project::is_project_version_dependency;
use crate::selection;

pub(crate) struct ApplyCommandRequest<'a> {
    pub(crate) input: DocumentInput,
    pub(crate) command: Option<&'a str>,
    pub(crate) dependency_name: Option<&'a str>,
    pub(crate) selected_version: Option<&'a str>,
    pub(crate) responses: &'a [RegistryResponseInput],
}

impl VersionLensSession {
    #[cfg(test)]
    pub(crate) async fn apply_command(
        &self,
        input: DocumentInput,
        command: Option<&str>,
        dependency_name: Option<&str>,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        self.apply_command_with_selected_version(ApplyCommandRequest {
            input,
            command,
            dependency_name,
            selected_version: None,
            responses,
        })
        .await
    }

    pub(crate) async fn apply_command_with_selected_version(
        &self,
        request: ApplyCommandRequest<'_>,
    ) -> ResolveDocumentOutput {
        let ApplyCommandRequest {
            input,
            command,
            dependency_name,
            selected_version,
            responses,
        } = request;
        if !recognized_apply_command(command) {
            return empty_resolve_output();
        }
        let operation = self.operation_context();
        if command == Some("sort") {
            let dependencies = self.dependencies(&input);
            let edits = sort_dependency_edits(&input.text, &dependencies);
            let parts = super::resolve_output_parts_with_plan(&input, &edits, 0, vec![], 0);
            return super::finish_resolve_output(vec![], parts);
        }
        let selected_version = selected_version.filter(|_| recognized_update_command(command));
        let plan_input = input.clone();

        let manifest_kind = self.classify_document(&input);
        let project_bump = project_version_bump(command, dependency_name);
        let mut suggestions = match dependency_name {
            Some(name) => {
                self.resolve_dependency_suggestions(DependencySuggestionsRequest {
                    input,
                    selector: name,
                    responses,
                    project_bump,
                    operation: &operation,
                })
                .await
            }
            None => {
                self.resolve_suggestions(input, responses, project_bump, &operation)
                    .await
            }
        };
        let bulk_dependency_update = bulk_dependency_update_command(command, dependency_name);
        if let Some(version) = selected_version {
            force_selected_version(&mut suggestions, version);
        }
        filter_update_command(&mut suggestions, command, selected_version.is_some());
        let edits = if bulk_dependency_update {
            let dependency_suggestions = suggestions
                .iter()
                .filter(|suggestion| !is_project_version_dependency(&suggestion.dependency))
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            bulk_update_edits(&dependency_suggestions)
        } else {
            update_edits(&suggestions)
        };
        let authorization_required_count = Self::authorization_required_count(&suggestions);
        let (vulnerable_update_count, vulnerable_update_package, vulnerable_update_version) =
            if self.config.show_vulnerabilities && dependency_name.is_some() {
                self.vulnerable_update_summary(
                    &suggestions,
                    responses,
                    Some(manifest_kind),
                    &operation,
                )
                .await
            } else {
                (0, None, None)
            };
        let authorization_required_requests = operation.take_authorization_requests();
        let authorization_required_count =
            authorization_required_count.max(to_u32(authorization_required_requests.len()));
        let mut parts = super::resolve_output_parts_with_plan(
            &plan_input,
            &edits,
            authorization_required_count,
            authorization_required_requests,
            vulnerable_update_count,
        );
        parts.vulnerable_update_package = vulnerable_update_package;
        parts.vulnerable_update_version = vulnerable_update_version;
        let graph = self.workspace_graph(&plan_input);
        let selected_dependency = dependency_name.and_then(|selector| {
            suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == selection::selector_name(selector))
                .map(|suggestion| &suggestion.dependency)
        });
        if !parts.edits.is_empty()
            && let Some(dependency) = selected_dependency
            && graph.resolve(dependency).is_some()
            && let Some(version) = selected_version
            && let Ok(Some(plan)) =
                graph.coordinated_plan(&plan_input, &parts.edits, &dependency.name, version)
        {
            parts.edit_plan = Some(plan);
            parts.edits.clear();
        }
        if !operation.can_publish() {
            return super::cancelled_resolution(suggestions);
        }
        super::finish_resolve_output(suggestions, parts)
    }

    pub(super) fn cached_update_vulnerabilities_deadline(
        &self,
        suggestion: &Suggestion,
        mut deadline: Instant,
    ) -> Instant {
        if !self.config.show_vulnerabilities {
            return deadline;
        }
        let now = Instant::now();
        let mut cache = self.vulnerability_cache();
        let mut include = |dependency: &Dependency| {
            deadline = deadline.min(
                cache
                    .expires_at(&vulnerability_cache_key(dependency))
                    .unwrap_or(now),
            );
        };
        include(&suggestion.dependency);
        if let Some(target) = target_update_dependency(suggestion) {
            include(&target);
        }
        for choice in &suggestion.choices {
            include(&update_dependency_for_version(suggestion, &choice.version));
        }
        deadline
    }

    pub(crate) async fn vulnerable_update_count(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
    ) -> u32 {
        self.vulnerable_update_summary(suggestions, responses, manifest_kind, operation)
            .await
            .0
    }

    async fn vulnerable_update_summary(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
    ) -> (u32, Option<String>, Option<String>) {
        if !self.config.show_vulnerabilities {
            return (0, None, None);
        }
        let mut dependencies = Vec::new();
        for suggestion in suggestions {
            dependencies.push(suggestion.dependency.clone());
            dependencies.extend(
                suggestion.choices.iter().map(|choice| {
                    update_dependency_for_version(suggestion, choice.version.as_str())
                }),
            );
            if let Some(target) = target_update_dependency(suggestion) {
                dependencies.push(target);
            }
        }
        let mut keys = HashSet::new();
        dependencies.retain(|dependency| keys.insert(vulnerability_cache_key(dependency)));
        let responses = Arc::new(responses.to_vec());
        let mut tasks = tokio::task::JoinSet::new();
        for dependency in dependencies {
            let session = self.clone();
            let responses = Arc::clone(&responses);
            let operation = operation.clone();
            tasks.spawn(async move {
                session
                    .cache_vulnerabilities(&dependency, &responses, manifest_kind, &operation)
                    .await;
            });
        }
        while let Some(result) = tasks.join_next().await {
            if let Err(error) = result {
                if error.is_panic() {
                    resume_unwind(error.into_panic());
                }
                return (0, None, None);
            }
        }

        let mut count = 0;
        let mut package = None;
        let mut version = None;
        for suggestion in suggestions {
            let Some(dependency) = target_update_dependency(suggestion) else {
                continue;
            };
            if !self.has_cached_vulnerabilities(&dependency) {
                continue;
            }
            count += 1;
            if package.is_none() {
                package = Some(dependency.name);
                version = Some(dependency.requirement);
            }
        }

        (to_u32(count), package, version)
    }

    pub(crate) fn target_update_has_cached_vulnerabilities(
        &self,
        suggestion: Option<&Suggestion>,
    ) -> bool {
        let Some(suggestion) = suggestion else {
            return false;
        };
        target_update_dependency(suggestion)
            .as_ref()
            .is_some_and(|dependency| self.has_cached_vulnerabilities(dependency))
    }

    pub(crate) fn authorization_required_count(suggestions: &[Suggestion]) -> u32 {
        let count = suggestions
            .iter()
            .filter(|suggestion| {
                suggestion.status == StatusError
                    && matches!(suggestion.latest.as_deref(), Some("401 not authorized"))
            })
            .count();
        to_u32(count)
    }
}

fn bulk_dependency_update_command(command: Option<&str>, dependency_name: Option<&str>) -> bool {
    dependency_name.is_none()
        && matches!(
            command,
            Some("update" | "updateMajor" | "updateMinor" | "updatePatch")
        )
}

fn recognized_apply_command(command: Option<&str>) -> bool {
    command.is_none() || command == Some("sort") || recognized_update_command(command)
}

fn recognized_update_command(command: Option<&str>) -> bool {
    matches!(
        command,
        Some(
            "update"
                | "updateMajor"
                | "updateMinor"
                | "updatePatch"
                | "updateRelease"
                | "updatePrerelease"
        )
    )
}

fn empty_resolve_output() -> ResolveDocumentOutput {
    ResolveDocumentOutput {
        suggestions: vec![],
        edits: vec![],
        authorization_required_count: 0,
        authorization_required_requests: vec![],
        vulnerable_update_count: 0,
        vulnerable_update_package: None,
        vulnerable_update_version: None,
        edit_plan: None,
    }
}

fn force_selected_version(suggestions: &mut [Suggestion], version: &str) {
    for suggestion in suggestions {
        if suggestion.dependency.is_runtime_constraint() {
            continue;
        }
        if suggestion.dependency.is_runtime_version()
            && !suggestion
                .choices
                .iter()
                .any(|choice| choice.version == version)
        {
            *suggestion = versionlens_update::error(
                suggestion.dependency.clone(),
                "selected runtime version has not been verified".to_owned(),
            );
            continue;
        }
        let requirement =
            versionlens_model::registry_alias_requirement(&suggestion.dependency.requirement)
                .unwrap_or(&suggestion.dependency.requirement);
        let dialect = if suggestion.dependency.ecosystem == versionlens_model::Ecosystem::Python {
            versionlens_model::VersionDialect::Pep440
        } else if suggestion.dependency.is_cargo_dependency_requirement() {
            versionlens_model::VersionDialect::Cargo
        } else {
            versionlens_model::VersionDialect::Semver
        };
        if versionlens_model::normalized_version_for_dialect(version, dialect).is_some()
            && versionlens_model::requirement_is_parseable_for_dialect(
                requirement,
                version,
                dialect,
            )
            && !versionlens_model::is_update_available_for_dialect(version, requirement, dialect)
            && !versionlens_model::requirement_satisfies_latest_for_dialect(
                requirement,
                version,
                dialect,
            )
        {
            suggestion.latest = None;
            suggestion.status = StatusError;
            suggestion.choices.clear();
            continue;
        }
        if suggestion.status == StatusFixed
            && suggestion.choices.is_empty()
            && suggestion.builds.is_empty()
        {
            continue;
        }
        if matches!(
            suggestion.status,
            StatusNotSupported
                | StatusDirectory
                | StatusDirectoryNotFound
                | StatusError
                | StatusRateLimited
                | StatusUnresolved
        ) {
            continue;
        }
        suggestion.latest = Some(version.to_owned());
        suggestion.status = StatusUpdateAvailable;
    }
}

fn target_update_dependency(suggestion: &Suggestion) -> Option<Dependency> {
    let latest = suggestion.latest.as_deref()?;
    (suggestion.status == StatusUpdateAvailable || suggestion.status == StatusBuildAvailable)
        .then(|| update_dependency_for_version(suggestion, latest))
}

fn update_dependency_for_version(suggestion: &Suggestion, version: &str) -> Dependency {
    Dependency {
        name: suggestion.dependency.name.as_str().to_owned(),
        requirement: version.to_owned(),
        ecosystem: suggestion.dependency.ecosystem,
        group: suggestion.dependency.group.as_str().to_owned(),
        hosted_url: suggestion
            .dependency
            .hosted_url
            .as_deref()
            .map(ToOwned::to_owned),
        hosted_name: suggestion
            .dependency
            .hosted_name
            .as_deref()
            .map(ToOwned::to_owned),
        range: suggestion.dependency.range,
        requirement_range: suggestion.dependency.requirement_range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

#[cfg(test)]
mod tests {
    use super::ApplyCommandRequest;
    use versionlens_model::{DocumentInput, Ecosystem};

    use crate::{
        DependencyPropertyConfig, ProviderSettings, RegistryResponseInput, SessionConfig,
        VersionLensSession,
    };

    fn standard_session() -> VersionLensSession {
        crate::test_support::test_session(true)
    }

    fn session_with_vulnerability_visibility(show_vulnerabilities: bool) -> VersionLensSession {
        let mut session = standard_session();
        session.config.show_vulnerabilities = show_vulnerabilities;
        session
    }

    fn session_with_dependency_properties(
        ecosystem: Ecosystem,
        properties: &[&str],
    ) -> VersionLensSession {
        crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings {
                dependency_properties: vec![DependencyPropertyConfig {
                    ecosystem,
                    manifest_kind: None,
                    properties: properties
                        .iter()
                        .map(|property| (*property).to_owned())
                        .collect(),
                }],
                ..Default::default()
            },
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: true,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        })
    }

    fn assert_single_named_edit(
        output: &crate::contract::ResolveDocumentOutput,
        name: &str,
        new_text: &str,
    ) {
        assert_eq!(output.suggestions.len(), 1);
        assert_eq!(output.suggestions[0].dependency.name, name);
        assert_eq!(output.edits.len(), 1);
        assert_eq!(output.edits[0].new_text, new_text);
    }

    mod command_contracts {
        use super::{ApplyCommandRequest, DocumentInput, RegistryResponseInput, standard_session};
        use crate::ResolveDocumentOutput;
        use versionlens_model::Ecosystem::{GitHub, Npm};

        #[tokio::test]
        async fn apply_command_updates_only_selected_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    package_input(r#"{"dependencies":{"left-pad":"1.0.0","is-odd":"2.0.0"}}"#),
                    None,
                    Some("left-pad"),
                    &[
                        npm_response("left-pad", "1.1.0"),
                        npm_response("is-odd", "3.0.0"),
                    ],
                )
                .await;

            super::assert_single_named_edit(&output, "left-pad", "1.1.0");
        }

        #[tokio::test]
        async fn apply_command_rejects_unknown_commands_without_resolving() {
            let session = standard_session();
            let input = package_input(r#"{"dependencies":{"left-pad":"1.0.0"}}"#);

            let output = session
                .apply_command(
                    input.clone(),
                    Some("executeArbitraryCommand"),
                    Some("left-pad"),
                    &[npm_response("left-pad", "9.9.9")],
                )
                .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
            assert_eq!(session.analyze_document(input).code_lenses.len(), 0);
        }

        #[tokio::test]
        async fn selected_version_without_update_command_is_ignored() {
            let session = standard_session();
            let response = RegistryResponseInput {
                package: "left-pad".to_owned(),
                ecosystem: Npm,
                body: r#"{"dist-tags":{"latest":"1.0.0+build.2"},"versions":{"1.0.0+build.2":{}}}"#
                    .to_owned(),
            };

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: package_input(r#"{"dependencies":{"left-pad":"1.0.0+build.1"}}"#),
                    command: None,
                    dependency_name: Some("left-pad"),
                    selected_version: Some("999.0.0"),
                    responses: &[response],
                })
                .await;

            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.suggestions.len(), 1);
            assert_ne!(output.suggestions[0].latest.as_deref(), Some("999.0.0"));
        }

        #[tokio::test]
        async fn unproven_selected_version_cannot_replace_a_sha_pin_with_a_mutable_tag() {
            let current = "7777777777777777777777777777777777777777";
            let output = apply_sha_action_selection(
                current,
                "999.0.0",
                "6666666666666666666666666666666666666666",
            )
            .await;

            assert_eq!(output.edits.len(), 0);
        }

        async fn apply_sha_action_selection(
            current: &str,
            selected: &str,
            older: &str,
        ) -> ResolveDocumentOutput {
            standard_session().apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            format!("steps:\n  - uses: actions/checkout@{current} # v7.0.1\n"),
            None,
        ),
        command: Some("update"),
        dependency_name: Some("actions/checkout"),
        selected_version: Some(selected),
        responses: &[RegistryResponseInput::new(
            "actions/checkout".to_owned(),
            GitHub,
            format!(r#"[{{"name":"v7.0.1","commit":{{"sha":"{current}"}}}},{{"name":"v6.0.0","commit":{{"sha":"{older}"}}}}]"#),
        )],
    }).await
        }

        fn package_input(text: &str) -> DocumentInput {
            DocumentInput {
                uri: "file:///package.json".to_owned(),
                language_id: "json".to_owned(),
                text: text.to_owned(),
                workspace_root: None,
                version: None,
            }
        }

        fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
            RegistryResponseInput {
                package: package.to_owned(),
                ecosystem: Npm,
                body: format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
            }
        }
    }
    mod sort {
        use super::{DocumentInput, session_with_dependency_properties, standard_session};
        use crate::VersionLensSession;
        use crate::contract::ResolveDocumentOutput;
        use versionlens_model::Ecosystem::*;
        use versionlens_model::TextEdit;

        async fn sort_fixture(
            session: &VersionLensSession,
            uri: &str,
            language: &str,
            fixture: &str,
        ) -> ResolveDocumentOutput {
            session
                .apply_command(
                    DocumentInput::new(
                        uri.to_owned(),
                        language.to_owned(),
                        package_file_fixture(fixture),
                        None,
                    ),
                    Some("sort"),
                    None,
                    &[],
                )
                .await
        }

        fn assert_sort_output(output: &ResolveDocumentOutput, expected: &[&str]) {
            assert_eq!(output.suggestions.len(), 0);
            if expected.is_empty() {
                assert_eq!(output.edits.len(), 0);
                return;
            }
            assert_eq!(output.edits.len(), expected.len());
            for (edit, expected_text) in output.edits.iter().zip(expected) {
                assert_eq!(edit.new_text, *expected_text);
            }
        }

        #[tokio::test]
        async fn apply_command_sorts_requirements_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///requirements.txt",
                "pip-requirements",
                "requirements-unsorted-with-comment.txt",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "alpha==1");
            assert_eq!(output.edits[1].new_text, "zeta==1");
        }

        #[tokio::test]
        async fn apply_command_sorts_smoke_requirements_dependencies() {
            let session = standard_session();

            let text = package_file_fixture("requirements-smoke.txt");
            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///requirements.txt".to_owned(),
                        "pip-requirements".to_owned(),
                        text.clone(),
                        None,
                    ),
                    Some("sort"),
                    None,
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(
                apply_line_edits(&text, &output.edits),
                "# Requirements for smoke testing\ndjango<=3.2\nflask>=2.0\nnot_found_package==1.17.0\nnumpy<1.22 # this should not cause issues\npandas~=1.2\npytest>3.0\npython-dateutil\nrequests==2.25.1\nsix==1.17.0\nurllib3===1.26.5"
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_pyproject_project_dependencies() {
            assert_sorted_pyproject_fixture(
                "pyproject-project-unsorted.toml",
                "[project]\ndependencies = [\n  \"alpha==1\",\n  \"zeta==1\"\n]",
            )
            .await;
        }

        #[tokio::test]
        async fn apply_command_sorts_pyproject_poetry_dependencies() {
            assert_sorted_pyproject_fixture(
                "pyproject-poetry-unsorted.toml",
                "[tool.poetry.dependencies]\nalpha = \"1\"\nzeta = \"1\"",
            )
            .await;
        }

        async fn assert_sorted_pyproject_fixture(fixture: &str, expected: &str) {
            let text = package_file_fixture(fixture);
            let output = standard_session()
                .apply_command(
                    DocumentInput::new(
                        "file:///pyproject.toml".to_owned(),
                        "toml".to_owned(),
                        text.clone(),
                        None,
                    ),
                    Some("sort"),
                    None,
                    &[],
                )
                .await;
            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(apply_line_edits(&text, &output.edits), expected);
        }

        #[tokio::test]
        async fn apply_command_sorts_pub_dependencies_by_group() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///pubspec.yaml",
                "yaml",
                "groups-unsorted.yaml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 4);
            assert_eq!(output.edits[0].new_text, "  alpha: 1");
            assert_eq!(output.edits[1].new_text, "  zeta: 1");
            assert_eq!(output.edits[2].new_text, "  a-dev: 1");
            assert_eq!(output.edits[3].new_text, "  z-dev: 1");
        }

        #[tokio::test]
        async fn apply_command_sorts_pub_dependencies_with_blank_versions() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///pubspec.yaml",
                "yaml",
                "blank-version.yaml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "  equatable:");
            assert_eq!(output.edits[1].new_text, "  flutter_bloc: 0.10.1");
        }

        #[tokio::test]
        async fn apply_command_sorts_complex_pub_dependencies() {
            let session = standard_session();

            let output =
                sort_fixture(&session, "file:///pubspec.yaml", "yaml", "complex.yaml").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "  equatable: ^0.2.0");
            assert_eq!(
                output.edits[1].new_text,
                "  sqflite:\n    git:\n      url: https://github.com/tekartik/sqflite\n      path: sqflite"
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_package_json_dependencies_by_group() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///package.json",
                "json",
                "groups-unsorted.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 4);
            assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
            assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
            assert_eq!(output.edits[2].new_text, "    \"a-dev\": \"1\",");
            assert_eq!(output.edits[3].new_text, "    \"z-dev\": \"1\"");
        }

        #[tokio::test]
        async fn apply_command_sorts_package_json_dependencies_with_metadata_entries() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///package.json",
                "json",
                "metadata-unsorted.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
            assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_docker_compose_images() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///docker-compose.yaml",
                "yaml",
                "docker-compose-images.yaml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_sorts_composer_require_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///composer.json",
                "json",
                "composer-require-unsorted.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "    \"allocine/twigcs\": \"^3.1.3\","
            );
            assert_eq!(
                output.edits[1].new_text,
                "    \"symfony/console\": \"8.1.*\""
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_deno_scoped_imports_within_each_scope() {
            let session = session_with_dependency_properties(Deno, &["scopes"]);

            let output = sort_fixture(
                &session,
                "file:///deno.json",
                "jsonc",
                "deno-scopes-unsorted.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 4);
            assert_eq!(
                output.edits[0].new_text,
                "      \"chalk\": \"npm:chalk@5.3.0\","
            );
            assert_eq!(
                output.edits[1].new_text,
                "      \"zeta\": \"npm:zeta@1.0.0\""
            );
            assert_eq!(
                output.edits[2].new_text,
                "      \"alpha\": \"jsr:@scope/alpha@1.0.0\","
            );
            assert_eq!(
                output.edits[3].new_text,
                "      \"bravo\": \"jsr:@scope/bravo@1.0.0\""
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_pnpm_named_catalog_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///pnpm-workspace.yaml",
                "yaml",
                "pnpm-workspace-named-catalog-unsorted.yaml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "    react: ^18.3.1");
            assert_eq!(output.edits[1].new_text, "    react-dom: ^19.2.7");
        }

        #[tokio::test]
        async fn apply_command_sorts_package_json_named_workspace_catalog_dependencies() {
            let session = session_with_dependency_properties(Npm, &["workspaces.catalogs.*"]);

            let output = sort_fixture(
                &session,
                "file:///package.json",
                "json",
                "workspace-catalog-unsorted.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "        \"react\": \"^18.3.1\",");
            assert_eq!(
                output.edits[1].new_text,
                "        \"react-dom\": \"^19.2.7\""
            );
        }
        #[tokio::test]
        async fn apply_command_sorts_maven_dependency_nodes() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///pom.xml",
                "xml",
                "dependencies-unsorted.xml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>"
            );
            assert_eq!(
                output.edits[1].new_text,
                "    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>"
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_configured_maven_dependency_management_nodes() {
            let session = session_with_dependency_properties(
                Maven,
                &["project.dependencyManagement.dependencies.dependency"],
            );

            let output = sort_fixture(
                &session,
                "file:///pom.xml",
                "xml",
                "dependency-management-unsorted.xml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "      <dependency>\n        <groupId>org.alpha</groupId>\n        <type>pom</type>\n        <artifactId>alpha</artifactId>\n        <version>1</version>\n      </dependency>"
            );
            assert_eq!(
                output.edits[1].new_text,
                "      <dependency>\n        <groupId>org.zeta</groupId>\n        <type>pom</type>\n        <artifactId>zeta</artifactId>\n        <version>1</version>\n      </dependency>"
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_maven_profile_dependency_nodes() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///pom.xml",
                "xml",
                "profile-dependencies-unsorted.xml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "        <dependency>\n          <groupId>org.alpha</groupId>\n          <scope>runtime</scope>\n          <artifactId>alpha</artifactId>\n          <version>1</version>\n        </dependency>"
            );
            assert_eq!(
                output.edits[1].new_text,
                "        <dependency>\n          <groupId>org.zeta</groupId>\n          <scope>runtime</scope>\n          <artifactId>zeta</artifactId>\n          <version>1</version>\n        </dependency>"
            );
        }

        #[tokio::test]
        async fn apply_command_sorts_dotnet_package_reference_tags() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///app.csproj",
                "xml",
                "app-package-references-unsorted.csproj",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "    <PackageReference Include=\"Alpha.Package\" Version=\"1\" />"
            );
            assert_eq!(
                output.edits[1].new_text,
                "    <PackageReference Include=\"Zeta.Package\" Version=\"1\" />"
            );
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_dotnet_package_references_across_item_groups() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///app.csproj",
                "xml",
                "app-package-references-item-groups.csproj",
            )
            .await;

            assert_sort_output(&output, &[]);
        }

        #[tokio::test]
        async fn apply_command_sorts_go_require_block_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///go.mod",
                "go.mod",
                "go-require-unsorted.mod",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(
                output.edits[0].new_text,
                "\talpha.example/pkg v1.0.0 // indirect"
            );
            assert_eq!(output.edits[1].new_text, "\tzeta.example/pkg v1.0.0");
        }

        #[tokio::test]
        async fn apply_command_sorts_go_exclude_block_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///go.mod",
                "go.mod",
                "go-exclude-unsorted.mod",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "\talpha.example/pkg v1.0.0");
            assert_eq!(output.edits[1].new_text, "\tzeta.example/pkg v1.0.0");
        }

        #[tokio::test]
        async fn apply_command_sorts_ruby_gemfile_dependencies() {
            let session = standard_session();

            let output =
                sort_fixture(&session, "file:///Gemfile", "ruby", "Gemfile-unsorted").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
            assert_eq!(output.edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_gleam_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///gleam.toml",
                "toml",
                "gleam-unsorted.toml",
            )
            .await;

            assert_sort_output(&output, &[]);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_gradle_build_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///build.gradle.kts",
                "kotlin",
                "build.gradle.kts",
            )
            .await;

            assert_sort_output(&output, &[]);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_sbt_library_dependencies() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///build.sbt", "scala", "build.sbt").await;

            assert_sort_output(&output, &[]);
        }
        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/commands/sort", name)
        }

        use std::cmp::Reverse;
        #[tokio::test]
        async fn apply_command_does_not_sort_clojure_deps_edn_dependencies() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///deps.edn", "clojure", "deps.edn").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_leiningen_project_clj_dependencies() {
            let session = standard_session();

            let output =
                sort_fixture(&session, "file:///project.clj", "clojure", "project.clj").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_terraform_required_providers() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///main.tf", "terraform", "main.tf").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_helm_chart_dependencies() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///Chart.yaml", "yaml", "Chart.yaml").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_ansible_requirements() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///work/requirements.yml",
                "yaml",
                "ansible-requirements.yml",
            )
            .await;

            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_bazel_modules() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///work/MODULE.bazel",
                "starlark",
                "MODULE.bazel",
            )
            .await;

            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_cocoapods_podfile_dependencies() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///work/Podfile", "ruby", "Podfile").await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_unity_project_manifest_dependencies() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///work/Packages/manifest.json",
                "json",
                "unity-manifest.json",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_kustomization_images() {
            let session = standard_session();

            let output = sort_fixture(
                &session,
                "file:///work/kustomization.yaml",
                "yaml",
                "kustomization.yaml",
            )
            .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_sort_nix_flake_inputs() {
            let session = standard_session();

            let output = sort_fixture(&session, "file:///work/flake.nix", "nix", "flake.nix").await;

            assert_eq!(output.edits.len(), 0);
        }

        fn apply_line_edits(text: &str, edits: &[TextEdit]) -> String {
            let mut lines: Vec<String> = text.lines().map(ToOwned::to_owned).collect();
            let mut ordered = edits.to_vec();
            ordered.sort_by_key(|edit| Reverse(edit.range.start.line));
            for edit in ordered {
                let start = usize::try_from(edit.range.start.line).unwrap();
                let end = usize::try_from(edit.range.end.line).unwrap();
                lines.splice(start..=end, edit.new_text.lines().map(ToOwned::to_owned));
            }
            lines.join("\n")
        }
    }
    mod update {
        use super::{
            ApplyCommandRequest, DocumentInput, session_with_vulnerability_visibility,
            standard_session,
        };
        use crate::RegistryResponseInput;
        use crate::contract::ResolveDocumentOutput;
        use versionlens_model::Ecosystem::*;

        #[tokio::test]
        async fn apply_command_updates_selected_build_version() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("command-updates-selected-build-version.json"),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("left-pad"),
                    selected_version: Some("1.0.0+build.3"),
                    responses: &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{
              "dist-tags": { "latest": "1.0.0+build.2" },
              "versions": {
                "1.0.0+build.1": {},
                "1.0.0+build.2": {},
                "1.0.0+build.3": {}
              }
            }"#
                        .to_owned(),
                    )],
                })
                .await;

            assert_single_edit(&output, "1.0.0+build.3");
        }

        #[tokio::test]
        async fn apply_command_updates_terraform_provider_version_without_replacing_operator() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///main.tf".to_owned(), "terraform".to_owned(), package_file_fixture(
                "command-updates-terraform-provider-version-without-replacing-operator.tf",
            ), None),
        Some("update"),
        Some("hashicorp/aws"),
        &[RegistryResponseInput::new("hashicorp/aws".to_owned(), Terraform, r#"{"versions":[{"version":"6.0.0"},{"version":"6.1.0-beta.1"}]}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "6.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_helm_chart_dependency_version_without_replacing_operator() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///Chart.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-helm-chart-dependency-version-without-replacing-operator.yaml"), None),
        Some("update"),
        Some("mysql"),
        &[RegistryResponseInput::new("mysql".to_owned(), Helm, "apiVersion: v1\nentries:\n  mysql:\n    - version: 4.0.0\n".to_owned())],
    ).await;

            assert_single_edit(&output, "4.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_ansible_collection_requirement_without_replacing_operator() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///work/requirements.yml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-ansible-collection-requirement-without-replacing-operator.yml"), None),
        Some("update"),
        Some("community.general"),
        &[RegistryResponseInput::new("community.general".to_owned(), AnsibleGalaxy, r#"{"data":[{"version":"8.0.0"},{"version":"7.5.0"}]}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "8.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_bazel_module_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/MODULE.bazel".to_owned(),
                        "starlark".to_owned(),
                        package_file_fixture("command-updates-bazel-module-dependencyMODULE.bazel"),
                        None,
                    ),
                    Some("update"),
                    Some("rules_cc"),
                    &[RegistryResponseInput::new(
                        "rules_cc".to_owned(),
                        Bazel,
                        r#"{"versions":["0.0.9","0.0.10"]}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "0.0.10");
        }

        #[tokio::test]
        async fn apply_command_updates_cocoapods_podfile_dependency_preserving_operator() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///work/Podfile".to_owned(), "ruby".to_owned(), package_file_fixture(
                "command-updates-cocoapods-podfile-dependency-preserving-operatorPodfile",
            ), None),
        Some("update"),
        Some("AFNetworking"),
        &[RegistryResponseInput::new("AFNetworking".to_owned(), CocoaPods, r#"{"versions":[{"name":"5.0.0"},{"name":"4.0.1"}]}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "~> 5.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_unity_project_manifest_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/Packages/manifest.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-updates-unity-project-manifest-dependency.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("com.unity.timeline"),
                    &[RegistryResponseInput::new(
                        "com.unity.timeline".to_owned(),
                        Unity,
                        r#"{"dist-tags":{"latest":"1.8.7"},"versions":{"1.8.6":{},"1.8.7":{}}}"#
                            .to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "1.8.7");
        }

        #[tokio::test]
        async fn apply_command_updates_kustomization_image_new_tag() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/kustomization.yaml".to_owned(),
                        "yaml".to_owned(),
                        package_file_fixture("command-updates-kustomization-image-new-tag.yaml"),
                        None,
                    ),
                    Some("update"),
                    Some("platform/nginx"),
                    &[RegistryResponseInput::new(
                        "platform/nginx".to_owned(),
                        Docker,
                        r#"{"tags":["1.26.0","1.25.3"]}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "1.26.0");
        }

        #[tokio::test]
        async fn apply_command_updates_nix_flake_github_input_ref() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/flake.nix".to_owned(),
                        "nix".to_owned(),
                        package_file_fixture("command-updates-nix-flake-github-input-ref.nix"),
                        None,
                    ),
                    Some("update"),
                    Some("NixOS/nixpkgs"),
                    &[RegistryResponseInput::new(
                        "NixOS/nixpkgs".to_owned(),
                        Nix,
                        r#"[{"name":"nixos-24.05"},{"name":"nixos-23.11"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "24.05");
        }
        fn assert_single_nonvulnerable_update(output: &ResolveDocumentOutput) {
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.vulnerable_update_count, 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_count_vulnerability_fixed_by_update() {
            let output = apply_vulnerability_case(VulnerabilityCase {
                fixture: "command-does-not-count-vulnerability-fixed-by-update.json",
                target: None,
                show_vulnerabilities: true,
                introduced: "0",
                fixed: "1.1.0",
                summary: "prototype issue",
                include_secondary: true,
            })
            .await;

            assert_eq!(output.suggestions.len(), 2);
            assert_eq!(output.edits.len(), 2);
            assert_eq!(output.authorization_required_count, 0);
            assert_eq!(output.vulnerable_update_count, 0);
        }

        #[tokio::test]
        async fn single_apply_command_counts_vulnerable_update_targets() {
            let output = apply_vulnerability_case(VulnerabilityCase {
                fixture: "single-apply-command-counts-vulnerable-update-targets.json",
                target: Some("left-pad"),
                show_vulnerabilities: true,
                introduced: "1.1.0",
                fixed: "2.0.0",
                summary: "target issue",
                include_secondary: true,
            })
            .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.authorization_required_count, 0);
            assert_eq!(output.vulnerable_update_count, 1);
            assert_eq!(
                output.vulnerable_update_package.as_deref(),
                Some("left-pad")
            );
            assert_eq!(output.vulnerable_update_version.as_deref(), Some("1.1.0"));
        }

        #[tokio::test]
        async fn bulk_apply_command_does_not_count_vulnerable_update_targets() {
            let output = apply_vulnerability_case(VulnerabilityCase {
                fixture: "bulk-apply-command-does-not-count-vulnerable-update-targets.json",
                target: None,
                show_vulnerabilities: true,
                introduced: "1.1.0",
                fixed: "2.0.0",
                summary: "target issue",
                include_secondary: false,
            })
            .await;

            assert_single_nonvulnerable_update(&output);
        }

        #[tokio::test]
        async fn single_apply_command_does_not_count_vulnerable_targets_when_vulnerabilities_are_hidden()
         {
            let output = apply_vulnerability_case(VulnerabilityCase {
        fixture: "single-apply-command-does-not-count-vulnerable-targets-when-vulnerabilities-are-hidden.json",
        target: Some("left-pad"),
        show_vulnerabilities: false,
        introduced: "1.1.0",
        fixed: "2.0.0",
        summary: "target issue",
        include_secondary: false,
    }).await;

            assert_single_nonvulnerable_update(&output);
        }

        #[tokio::test]
        async fn apply_command_counts_authorization_required_failures() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("command-counts-authorization-required-failures.json"),
                        None,
                    ),
                    Some("update"),
                    None,
                    &[RegistryResponseInput::new(
                        "private-package".to_owned(),
                        Npm,
                        r#"{"status":401}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.authorization_required_count, 1);
            assert_eq!(output.vulnerable_update_count, 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_count_forbidden_registry_failures_as_authorization_required()
         {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("command-does-not-count-forbidden-registry-failures-as-authorization-required.json"), None),
        Some("update"),
        None,
        &[RegistryResponseInput::new("private-package".to_owned(), Npm, r#"{"status":403}"#.to_owned())],
    ).await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.authorization_required_count, 0);
            assert_eq!(output.vulnerable_update_count, 0);
        }
        struct VulnerabilityCase<'a> {
            fixture: &'a str,
            target: Option<&'a str>,
            show_vulnerabilities: bool,
            introduced: &'a str,
            fixed: &'a str,
            summary: &'a str,
            include_secondary: bool,
        }

        async fn apply_vulnerability_case(case: VulnerabilityCase<'_>) -> ResolveDocumentOutput {
            let session = if case.show_vulnerabilities {
                standard_session()
            } else {
                session_with_vulnerability_visibility(false)
            };
            let vulnerability = serde_json::json!({
                "dist-tags": {"latest": "1.1.0"},
                "vulns": [{
                    "id": "OSV-1",
                    "summary": case.summary,
                    "affected": [{
                        "package": {"name": "left-pad"},
                        "ranges": [{"events": [{"introduced": case.introduced}, {"fixed": case.fixed}]}]
                    }]
                }]
            });
            let mut responses = vec![RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                vulnerability.to_string(),
            )];
            if case.include_secondary {
                responses.push(RegistryResponseInput::new(
                    "is-odd".to_owned(),
                    Npm,
                    r#"{"dist-tags":{"latest":"3.0.0"}}"#.to_owned(),
                ));
            }
            session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(case.fixture),
                        None,
                    ),
                    Some("update"),
                    case.target,
                    &responses,
                )
                .await
        }
        #[tokio::test]
        async fn apply_command_uses_code_lens_selector_for_duplicate_names() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("command-uses-code-lens-selector-for-duplicate-names.json"),
                None,
            );

            let responses = [RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
            )];
            session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            let command_input = input.clone();
            let analyzed = session.analyze_document(input);
            let selector = analyzed
                .code_lenses
                .iter()
                .find(|lens| lens.action_kind() == "UpdateDependency")
                .and_then(|lens| lens.arguments().get(1).cloned())
                .expect("update code lens selector");
            let output = session
                .apply_command(command_input, None, Some(&selector), &responses)
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, "dependencies");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "1.1.0");
        }

        #[tokio::test]
        async fn pyproject_ranges_resolving_latest_do_not_offer_noop_lower_bound_bumps() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///pyproject.toml".to_owned(),
                "toml".to_owned(),
                package_file_fixture(
                    "pyproject-update-code-lenses-advance-lower-bounds-and-preserve-upper-caps.toml",
                ),
                None,
            );
            let responses = [
        RegistryResponseInput::new("httpx".to_owned(), Python, r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned()),
        RegistryResponseInput::new("httpcore".to_owned(), Python, r#"{"info":{"version":"0.28.1"},"releases":{"0.27.0":[],"0.28.1":[{"yanked":false}]}}"#
                .to_owned()),
    ];

            session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            let analyzed = session.analyze_document(input);

            assert_eq!(
                analyzed
                    .code_lenses
                    .iter()
                    .map(|lens| (lens.title.as_str(), lens.action_kind()))
                    .collect::<Vec<_>>(),
                [
                    ("🟢 satisfies latest 0.28.1", ""),
                    ("🟢 satisfies latest 0.28.1", "")
                ]
            );
        }

        #[tokio::test]
        async fn pyproject_selected_pep440_updates_preserve_or_repair_extended_bounds() {
            for (package, requirement, provider_latest, selected, expected) in [
                (
                    "epoch-package",
                    ">=1!1.0,<1!2.0",
                    "1!1.5",
                    "1!1.5",
                    ">=1!1.5, <1!2.0",
                ),
                (
                    "post-package",
                    ">=1.0.post1,<1.0.post3",
                    "1.0.post2",
                    "1.0.post2",
                    ">=1.0.post2, <1.0.post3",
                ),
                (
                    "dev-package",
                    ">=1.0.dev1,<1.0",
                    "1.0",
                    "1.0.dev2",
                    ">=1.0.dev2, <=1.0.dev2",
                ),
                (
                    "local-package",
                    ">=1.0,<2.0,!=1.5+linux",
                    "1.5+mac",
                    "1.5+mac",
                    ">=1.5, <2.0, !=1.5+linux",
                ),
            ] {
                let session = standard_session();
                let input = DocumentInput::new(
                    "file:///pyproject.toml".to_owned(),
                    "toml".to_owned(),
                    format!("[project]\ndependencies = [\"{package}{requirement}\"]\n"),
                    None,
                );
                let responses = [RegistryResponseInput::new(
                    package.to_owned(),
                    Python,
                    format!(
                        r#"{{"info":{{"version":"{provider_latest}"}},"releases":{{"{provider_latest}":[{{"yanked":false}}]}}}}"#,
                    ),
                )];

                let output = session
                    .apply_command_with_selected_version(ApplyCommandRequest {
                        input,
                        command: Some("update"),
                        dependency_name: Some(package),
                        selected_version: Some(selected),
                        responses: &responses,
                    })
                    .await;

                assert_eq!(output.suggestions.len(), 1, "{package}");
                assert_eq!(
                    output.suggestions[0].dependency.requirement, requirement,
                    "{package}",
                );
                assert_eq!(output.edits.len(), 1, "{package}");
                assert_eq!(output.edits[0].new_text, expected, "{package}");
            }
        }

        #[tokio::test]
        async fn apply_command_updates_only_requested_level() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("command-updates-only-requested-level.json"),
                        None,
                    ),
                    Some("updateMinor"),
                    None,
                    &[
                        RegistryResponseInput::new(
                            "major".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
                        ),
                        RegistryResponseInput::new(
                            "minor".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
                        ),
                        RegistryResponseInput::new(
                            "patch".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"1.0.1"}}"#.to_owned(),
                        ),
                    ],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.name, "minor");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "1.1.0");
        }

        #[tokio::test]
        async fn apply_command_updates_ranged_dependency_to_requested_minor_choice() {
            assert_ranged_choice(
                "command-updates-ranged-dependency-to-requested-minor-choice.json",
                "updateMinor",
                "~1.1.0",
            )
            .await;
        }

        #[tokio::test]
        async fn apply_command_updates_ranged_dependency_to_requested_patch_choice() {
            assert_ranged_choice(
                "command-updates-ranged-dependency-to-requested-patch-choice.json",
                "updatePatch",
                "1.0.1",
            )
            .await;
        }

        async fn assert_ranged_choice(fixture: &str, command: &str, expected: &str) {
            let output = standard_session().apply_command(
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some(command),
        None,
        &[RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"1.0.1":{},"1.1.0":{},"2.0.0":{}}}"#.to_owned(),
        )],
    ).await;
            super::assert_single_named_edit(&output, "left-pad", expected);
        }

        #[tokio::test]
        async fn apply_command_level_filter_does_not_bump_project_version() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-level-filter-does-not-bump-project-version.json",
                        ),
                        None,
                    ),
                    Some("updateMajor"),
                    None,
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.name, "left-pad");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_bulk_update_skips_project_version_edits() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-bulk-update-skips-project-version-edits.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    None,
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn bulk_update_skips_prerelease_only_invalid_range_updates() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "bulk-update-skips-prerelease-only-invalid-range-updates.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    None,
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{
              "dist-tags": { "latest": "5.0.0-beta.1" },
              "versions": {
                "5.0.0-beta.1": {}
              }
            }"#
                        .to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].status, "invalidRange");
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn bulk_update_raises_the_floor_of_a_caret_range_that_allows_latest() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        r#"{"dependencies":{"left-pad":"^1.0.0"}}"#.to_owned(),
                        None,
                    ),
                    Some("update"),
                    None,
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"1.3.0"},"versions":{"1.0.0":{},"1.3.0":{}}}"#
                            .to_owned(),
                    )],
                )
                .await;

            assert_eq!(
                output.edits.len(),
                1,
                "suggestions: {:?}",
                output.suggestions
            );
            assert_eq!(output.edits[0].new_text, "^1.3.0");
        }

        #[tokio::test]
        async fn bulk_update_raises_the_floor_of_a_cargo_bare_requirement() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///Cargo.toml".to_owned(), "toml".to_owned(), "[dependencies]\nserde = \"1.0\"\n".to_owned(), None),
        Some("update"),
        None,
        &[RegistryResponseInput::new("serde".to_owned(), Cargo, r#"{"versions":[{"num":"1.0.0","yanked":false},{"num":"1.1.0","yanked":false}]}"#.to_owned())],
    ).await;

            assert_eq!(
                output.edits.len(),
                1,
                "suggestions: {:?}",
                output.suggestions
            );
            assert_eq!(output.edits[0].new_text, "1.1.0");
        }

        #[tokio::test]
        async fn bulk_update_leaves_an_exact_pin_at_latest_alone() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        r#"{"dependencies":{"left-pad":"1.3.0"}}"#.to_owned(),
                        None,
                    ),
                    Some("update"),
                    None,
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"1.3.0"},"versions":{"1.0.0":{},"1.3.0":{}}}"#
                            .to_owned(),
                    )],
                )
                .await;

            assert!(output.edits.is_empty(), "edits: {:?}", output.edits);
        }
        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/commands/update", name)
        }

        fn assert_single_edit(output: &ResolveDocumentOutput, expected: &str) {
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, expected);
        }

        fn assert_single_dependency_update(
            output: &ResolveDocumentOutput,
            group: &str,
            current: &str,
            selected: &str,
        ) {
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, group);
            assert_eq!(output.suggestions[0].dependency.requirement, current);
            assert_single_edit(output, selected);
        }

        fn assert_project_update(output: &ResolveDocumentOutput, name: &str, expected: &str) {
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, "version");
            assert_eq!(output.suggestions[0].dependency.name, name);
            assert_single_edit(output, expected);
        }

        fn assert_project_edit(output: &ResolveDocumentOutput, expected: &str) {
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, "version");
            assert_single_edit(output, expected);
        }

        #[tokio::test]
        async fn apply_command_preserves_semver_requirement_prefix() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("command-preserves-semver-requirement-prefix.json"),
                        None,
                    ),
                    Some("update"),
                    Some("left-pad"),
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "^2.0.0");
        }

        #[tokio::test]
        async fn apply_command_preserves_composer_stability_flag_suffix() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///composer.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-preserves-composer-stability-flag-suffix.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("monolog/monolog"),
                    &[RegistryResponseInput::new(
                        "monolog/monolog".to_owned(),
                        Composer,
                        r#"{"packages":{"monolog/monolog":[{"version":"1.1.0"}]}}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "1.1.0@beta");
        }

        #[tokio::test]
        async fn apply_command_updates_project_version_by_requested_level() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-updates-project-version-by-requested-level.json",
                        ),
                        None,
                    ),
                    Some("updateMajor"),
                    Some("1.2.3"),
                    &[],
                )
                .await;

            assert_project_edit(&output, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_jsr_project_version_by_requested_level() {
            assert_project_version_update(
                "file:///jsr.json",
                "json",
                "command-updates-jsr-project-version-by-requested-level.json",
                "@scope/pkg",
            )
            .await;
        }

        #[tokio::test]
        async fn apply_command_updates_deno_json_jsr_project_version_by_requested_level() {
            assert_project_version_update(
                "file:///deno.json",
                "jsonc",
                "command-updates-deno-json-jsr-project-version-by-requested-level.json",
                "@scope/pkg",
            )
            .await;
        }

        async fn assert_project_version_update(
            uri: &str,
            language_id: &str,
            fixture: &str,
            name: &str,
        ) {
            let output = standard_session()
                .apply_command(
                    DocumentInput::new(
                        uri.to_owned(),
                        language_id.to_owned(),
                        package_file_fixture(fixture),
                        None,
                    ),
                    Some("updatePatch"),
                    Some(name),
                    &[],
                )
                .await;
            assert_project_update(&output, name, "1.2.4");
        }

        #[tokio::test]
        async fn apply_command_updates_prerelease_project_version_by_requested_level() {
            assert_prerelease_project_version_update(
                "command-updates-prerelease-project-version-by-requested-level.json",
                Some("updateRelease"),
                Some("1.2.3-beta.4"),
                "1.2.3",
            )
            .await;
        }

        #[tokio::test]
        async fn apply_command_updates_only_project_versions_for_prerelease_command() {
            assert_prerelease_project_version_update(
                "command-updates-only-project-versions-for-prerelease-command.json",
                Some("updatePrerelease"),
                None,
                "1.2.3-beta.5",
            )
            .await;
        }

        async fn assert_prerelease_project_version_update(
            fixture: &str,
            command: Option<&str>,
            selected_version: Option<&str>,
            expected: &str,
        ) {
            let output = standard_session()
                .apply_command(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(fixture),
                        None,
                    ),
                    command,
                    selected_version,
                    &[crate::test_support::npm_latest_response(
                        "left-pad", "2.0.0",
                    )],
                )
                .await;
            assert_project_edit(&output, expected);
        }

        #[tokio::test]
        async fn apply_command_updates_cargo_project_version_by_requested_level() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///Cargo.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "command-updates-cargo-project-version-by-requested-level.toml",
                        ),
                        None,
                    ),
                    Some("updatePatch"),
                    Some("version"),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, "package");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "1.2.4");
        }

        #[tokio::test]
        async fn apply_command_updates_cargo_renamed_package_version_preserving_alias() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///Cargo.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "command-updates-cargo-renamed-package-version-preserving-alias.toml",
                        ),
                        None,
                    ),
                    None,
                    Some("local_name"),
                    &[RegistryResponseInput::new(
                        "registry-name".to_owned(),
                        Cargo,
                        r#"{"versions":[{"num":"2.0.0"}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.name, "local_name");
            assert_eq!(
                output.suggestions[0].dependency.hosted_name.as_deref(),
                Some("registry-name")
            );
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_go_hyphenated_prerelease_version() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///go.mod".to_owned(),
                        "go.mod".to_owned(),
                        package_file_fixture(
                            "command-updates-go-hyphenated-prerelease-version.mod",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("example.test/prerelease"),
                    selected_version: Some("v1.0.0"),
                    responses: &[RegistryResponseInput::new(
                        "example.test/prerelease".to_owned(),
                        Go,
                        "v1.0.0-alpha-beta\nv1.0.0\n".to_owned(),
                    )],
                })
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(
                output.suggestions[0].dependency.requirement,
                "v1.0.0-alpha-beta"
            );
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "v1.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_bare_requirements_with_equals_prefix() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///requirements.txt".to_owned(),
                        "pip-requirements".to_owned(),
                        package_file_fixture(
                            "command-updates-bare-requirements-with-equals-prefix.txt",
                        ),
                        None,
                    ),
                    None,
                    Some("importlib-metadata"),
                    &[RegistryResponseInput::new(
                        "importlib-metadata".to_owned(),
                        Python,
                        r#"{"info":{"version":"8.7.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "==8.7.0");
        }

        #[tokio::test]
        async fn apply_command_updates_empty_pipfile_requirements_with_equals_prefix() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///Pipfile".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "command-updates-empty-pipfile-requirements-with-equals-prefix.Pipfile",
                        ),
                        None,
                    ),
                    None,
                    Some("magic"),
                    &[RegistryResponseInput::new(
                        "magic".to_owned(),
                        Python,
                        r#"{"info":{"version":"1.2.3"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "==1.2.3");
        }

        #[tokio::test]
        async fn apply_command_inserts_missing_deno_import_versions() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///deno.json".to_owned(),
                        "jsonc".to_owned(),
                        package_file_fixture("command-inserts-missing-deno-import-versions.json"),
                        None,
                    ),
                    None,
                    Some("@std/assert"),
                    &[RegistryResponseInput::new(
                        "@std/assert".to_owned(),
                        Deno,
                        r#"{"versions":{"1.0.1":{}}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.0.1");
        }
        #[tokio::test]
        async fn apply_command_updates_dotnet_package_reference_child_version() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///app.csproj".to_owned(),
                        "xml".to_owned(),
                        package_file_fixture(
                            "command-updates-dotnet-package-reference-child-version.csproj",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("Microsoft.NET.Test.Sdk"),
                    selected_version: Some("18.8.0"),
                    responses: &[RegistryResponseInput::new(
                        "Microsoft.NET.Test.Sdk".to_owned(),
                        Dotnet,
                        r#"{"versions":["18.7.0","18.8.0"]}"#.to_owned(),
                    )],
                })
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.requirement, "18.7.0");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "18.8.0");
        }

        #[tokio::test]
        async fn apply_command_updates_gradle_groovy_dependency_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.gradle".to_owned(), "groovy".to_owned(), package_file_fixture("command-updates-gradle-groovy-dependency-version.gradle"), None),
        command: Some("update"),
        dependency_name: Some("org.springframework:spring-core"),
        selected_version: Some("6.2.9"),
        responses: &[RegistryResponseInput::new("org.springframework:spring-core".to_owned(), Maven, r"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "implementation", "6.2.8", "6.2.9");
        }

        #[tokio::test]
        async fn apply_command_updates_gradle_kotlin_named_argument_dependency_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.gradle.kts".to_owned(), "kotlin".to_owned(), package_file_fixture("command-updates-gradle-kotlin-named-argument-dependency-version.gradle.kts"), None),
        command: Some("update"),
        dependency_name: Some("org.slf4j:slf4j-api"),
        selected_version: Some("2.0.18"),
        responses: &[RegistryResponseInput::new("org.slf4j:slf4j-api".to_owned(), Maven, r"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "implementation", "2.0.17", "2.0.18");
        }

        #[tokio::test]
        async fn apply_command_updates_gradle_plugin_dsl_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///settings.gradle.kts".to_owned(), "kotlin".to_owned(), package_file_fixture("command-updates-gradle-plugin-dsl-version.gradle.kts"), None),
        command: Some("update"),
        dependency_name: Some("com.android.application:com.android.application.gradle.plugin"),
        selected_version: Some("8.12.1"),
        responses: &[RegistryResponseInput::new("com.android.application:com.android.application.gradle.plugin".to_owned(), Maven, r"<metadata><versioning><versions><version>8.12.0</version><version>8.12.1</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "plugins", "8.12.0", "8.12.1");
        }

        #[tokio::test]
        async fn apply_command_updates_gradle_version_catalog_library_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///gradle/libs.versions.toml".to_owned(), "toml".to_owned(), package_file_fixture("command-updates-gradle-version-catalog-library-version.versions.toml"), None),
        command: Some("update"),
        dependency_name: Some("org.apache.commons:commons-lang3"),
        selected_version: Some("3.18.0"),
        responses: &[RegistryResponseInput::new("org.apache.commons:commons-lang3".to_owned(), Maven, r"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "libraries", "3.17.0", "3.18.0");
        }

        #[tokio::test]
        async fn apply_command_updates_gradle_version_catalog_plugin_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///gradle/libs.versions.toml".to_owned(), "toml".to_owned(), package_file_fixture("command-updates-gradle-version-catalog-plugin-version.versions.toml"), None),
        command: Some("update"),
        dependency_name: Some("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"),
        selected_version: Some("0.52.0"),
        responses: &[RegistryResponseInput::new("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(), Maven, r"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "plugins", "0.51.0", "0.52.0");
        }

        #[tokio::test]
        async fn apply_command_updates_sbt_library_dependency_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-updates-sbt-library-dependency-version.sbt"), None),
        command: Some("update"),
        dependency_name: Some("org.scala-stm:scala-stm_2.13"),
        selected_version: Some("0.9.2"),
        responses: &[RegistryResponseInput::new("org.scala-stm:scala-stm_2.13".to_owned(), Maven, r"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(
                output.suggestions[0].dependency.group,
                "libraryDependencies"
            );
            assert_eq!(output.suggestions[0].dependency.requirement, "0.9.1");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "0.9.2");
        }

        #[tokio::test]
        async fn apply_command_updates_sbt_dependency_override_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-updates-sbt-dependency-override-version.sbt"), None),
        command: Some("update"),
        dependency_name: Some("log4j:log4j"),
        selected_version: Some("1.2.17"),
        responses: &[RegistryResponseInput::new("log4j:log4j".to_owned(), Maven, r"<metadata><versioning><versions><version>1.2.16</version><version>1.2.17</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(
                output.suggestions[0].dependency.group,
                "dependencyOverrides"
            );
            assert_eq!(output.suggestions[0].dependency.requirement, "1.2.16");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "1.2.17");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_sbt_url_artifact_dependency() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-does-not-update-sbt-url-artifact-dependency.sbt"), None),
        command: Some("update"),
        dependency_name: Some("slinky:slinky"),
        selected_version: Some("2.2"),
        responses: &[RegistryResponseInput::new("slinky:slinky".to_owned(), Maven, r"<metadata><versioning><versions><version>2.1</version><version>2.2</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_eq!(output.suggestions.len(), 1);
            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "fixed",
                Some("package URL"),
            );
        }

        #[tokio::test]
        async fn apply_command_updates_clojure_deps_edn_maven_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///deps.edn".to_owned(), "clojure".to_owned(), package_file_fixture("command-updates-clojure-deps-edn-maven-version.edn"), None),
        command: Some("update"),
        dependency_name: Some("org.clojure:tools.reader"),
        selected_version: Some("1.2.0"),
        responses: &[RegistryResponseInput::new("org.clojure:tools.reader".to_owned(), Maven, r"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "deps", "1.1.1", "1.2.0");
        }

        #[tokio::test]
        async fn apply_command_updates_leiningen_project_clj_dependency_version() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///project.clj".to_owned(), "clojure".to_owned(), package_file_fixture("command-updates-leiningen-project-clj-dependency-version.clj"), None),
        command: Some("update"),
        dependency_name: Some("org.clojure:clojure"),
        selected_version: Some("1.12.0"),
        responses: &[RegistryResponseInput::new("org.clojure:clojure".to_owned(), Maven, r"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>".to_owned())],
    }).await;

            assert_single_dependency_update(&output, "dependencies", "1.11.3", "1.12.0");
        }

        #[tokio::test]
        async fn apply_command_updates_packages_config_version_attribute() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///packages.config".to_owned(),
                        "xml".to_owned(),
                        package_file_fixture(
                            "command-updates-packages-config-version-attribute.config",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("jQuery"),
                    selected_version: Some("3.7.1"),
                    responses: &[RegistryResponseInput::new(
                        "jQuery".to_owned(),
                        Dotnet,
                        r#"{"versions":["3.1.1","3.7.1"]}"#.to_owned(),
                    )],
                })
                .await;

            assert_single_dependency_update(&output, "packages.package", "3.1.1", "3.7.1");
        }
        #[tokio::test]
        async fn apply_command_updates_deno_jsr_import_aliases_by_specifier_package() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///deno.json".to_owned(),
                        "jsonc".to_owned(),
                        package_file_fixture(
                            "command-updates-deno-jsr-import-aliases-by-specifier-package.json",
                        ),
                        None,
                    ),
                    None,
                    Some("luca"),
                    &[RegistryResponseInput::new(
                        "@luca/cases".to_owned(),
                        Deno,
                        r#"{"versions":{"1.1.0":{},"1.0.0":{}}}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "jsr:@luca/cases@1.1.0");
        }

        #[tokio::test]
        async fn apply_command_updates_import_map_directory_specifier_preserving_slashes() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///import_map.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-updates-import-map-directory-specifier-preserving-slashes.json",
            ), None),
        None,
        Some("@std/async/"),
        &[RegistryResponseInput::new("@std/async".to_owned(), Deno, r#"{"versions":{"2.0.0":{},"1.0.0":{}}}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "jsr:/@std/async@^2.0.0/");
        }

        #[tokio::test]
        async fn apply_command_updates_conanfile_txt_dependency_preserving_revision_suffix() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///conanfile.txt".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-conanfile-txt-dependency-preserving-revision-suffix.txt",
            ), None),
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput::new("zlib".to_owned(), Conan, r#"{"results":["zlib/1.2.13","zlib/1.3.1"]}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "1.3.1#rev0");
        }

        #[tokio::test]
        async fn apply_command_updates_stack_resolver_from_stackage_snapshot_index() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///stack.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-stack-resolver-from-stackage-snapshot-index.yaml"), None),
        Some("update"),
        Some("stackage-lts"),
        &[RegistryResponseInput::new("stackage-lts".to_owned(), Hackage, r#"{"snapshots":[[["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "24.49");
        }

        #[tokio::test]
        async fn apply_command_updates_gemspec_dependency_preserving_ruby_operator() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///example.gemspec".to_owned(),
                        "ruby".to_owned(),
                        package_file_fixture(
                            "command-updates-gemspec-dependency-preserving-ruby-operator.gemspec",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("rack"),
                    &[RegistryResponseInput::new(
                        "rack".to_owned(),
                        Ruby,
                        r#"[{"number":"3.0.0"},{"number":"2.2.9"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "~> 3.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_pub_hosted_dependency_without_version_by_inserting_version()
        {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///pubspec.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-pub-hosted-dependency-without-version-by-inserting-version.yaml"), None),
        Some("update"),
        Some("hosted_dep"),
        &[RegistryResponseInput::new("hosted_alias".to_owned(), Pub, r#"{"latest":{"version":"2.0.0"}}"#.to_owned())],
    ).await;

            assert_single_edit(&output, "\n    version: 2.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_gleam_project_version_by_requested_level() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///gleam.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "command-updates-gleam-project-version-by-requested-level.toml",
                        ),
                        None,
                    ),
                    Some("updateMinor"),
                    Some("my_package"),
                    &[],
                )
                .await;

            assert_project_update(&output, "my_package", "1.3.0");
        }

        #[tokio::test]
        async fn apply_command_updates_gleam_dependency_preserving_requirement_syntax() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///gleam.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "command-updates-gleam-dependency-preserving-requirement-syntax.toml",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("gleam_stdlib"),
                    &[RegistryResponseInput::new(
                        "gleam_stdlib".to_owned(),
                        Hex,
                        r#"{"releases":[{"version":"2.0.0"},{"version":"0.44.0"}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_rebar_dependency_version() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///rebar.config".to_owned(),
                        "erlang".to_owned(),
                        package_file_fixture("command-updates-rebar-dependency-version.config"),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("cowboy"),
                    selected_version: Some("2.13.0"),
                    responses: &[RegistryResponseInput::new(
                        "cowboy".to_owned(),
                        Hex,
                        r#"{"releases":[{"version":"2.13.0"},{"version":"2.12.0"}]}"#.to_owned(),
                    )],
                })
                .await;
            assert_single_edit(&output, "2.13.0");
        }

        #[tokio::test]
        async fn apply_command_updates_opam_dependency_preserving_operator() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///demo.opam".to_owned(),
                        "plaintext".to_owned(),
                        package_file_fixture(
                            "command-updates-opam-dependency-preserving-operator.opam",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("lwt"),
                    selected_version: Some("6.1.2"),
                    responses: &[RegistryResponseInput::new(
                        "lwt".to_owned(),
                        Opam,
                        r"<h2>lwt version</h2><p>6.1.2 (latest)</p>".to_owned(),
                    )],
                })
                .await;
            assert_single_edit(&output, ">= \"6.1.2\"");
        }

        #[tokio::test]
        async fn apply_command_updates_dune_project_dependency_preserving_operator() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///dune-project".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-dune-project-dependency-preserving-operator.dune-project",
            ), None),
        command: Some("update"),
        dependency_name: Some("fmt"),
        selected_version: Some("0.9.0"),
        responses: &[RegistryResponseInput::new("fmt".to_owned(), Opam, r"<h2>fmt version</h2><p>0.9.0 (latest)</p>".to_owned())],
    }).await;
            assert_single_edit(&output, ">= 0.9.0");
        }

        #[tokio::test]
        async fn apply_command_updates_cabal_dependency_preserving_operator() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///demo.cabal".to_owned(),
                        "plaintext".to_owned(),
                        package_file_fixture(
                            "command-updates-cabal-dependency-preserving-operator.cabal",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("base"),
                    selected_version: Some("4.20.0.0"),
                    responses: &[RegistryResponseInput::new(
                        "base".to_owned(),
                        Hackage,
                        r#"{"4.20.0.0":"normal","4.19.2.0":"normal"}"#.to_owned(),
                    )],
                })
                .await;
            assert_single_edit(&output, ">= 4.20.0.0");
        }
        #[tokio::test]
        async fn apply_command_updates_julia_compat_dependency() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///Project.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture("command-updates-julia-compat-dependency.toml"),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("Example"),
                    selected_version: Some("0.6.0"),
                    responses: &[RegistryResponseInput::new(
                        "Example".to_owned(),
                        Julia,
                        r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0]
git-tree-sha1 = "d6f6"
"#
                        .to_owned(),
                    )],
                })
                .await;
            assert_single_edit(&output, "0.6.0");
        }

        #[tokio::test]
        async fn apply_command_updates_r_description_dependency_preserving_operator() {
            let session = standard_session();

            let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///DESCRIPTION".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-r-description-dependency-preserving-operatorDESCRIPTION",
            ), None),
        command: Some("update"),
        dependency_name: Some("dplyr"),
        selected_version: Some("1.1.4"),
        responses: &[RegistryResponseInput::new("dplyr".to_owned(), Cran, "Package: dplyr\nVersion: 1.1.4\n".to_owned())],
    }).await;
            assert_single_edit(&output, ">= 1.1.4");
        }

        #[tokio::test]
        async fn apply_command_updates_paket_dependencies_nuget_version() {
            let session = standard_session();

            let output = session
                .apply_command_with_selected_version(ApplyCommandRequest {
                    input: DocumentInput::new(
                        "file:///paket.dependencies".to_owned(),
                        "plaintext".to_owned(),
                        package_file_fixture(
                            "command-updates-paket-dependencies-nuget-version.dependencies",
                        ),
                        None,
                    ),
                    command: Some("update"),
                    dependency_name: Some("Newtonsoft.Json"),
                    selected_version: Some("13.0.3"),
                    responses: &[RegistryResponseInput::new(
                        "Newtonsoft.Json".to_owned(),
                        Dotnet,
                        r#"{"versions":["13.0.1","13.0.3"]}"#.to_owned(),
                    )],
                })
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].dependency.group, "paket.dependencies");
            assert_eq!(output.suggestions[0].dependency.requirement, "13.0.1");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "13.0.3");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_paket_references_without_versions() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///paket.references".to_owned(),
                        "plaintext".to_owned(),
                        package_file_fixture(
                            "command-does-not-update-paket-references-without-versions.references",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("Newtonsoft.Json"),
                    &[RegistryResponseInput::new(
                        "Newtonsoft.Json".to_owned(),
                        Dotnet,
                        r#"{"versions":["13.0.3"]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_does_not_update_dockerfile_digest_pinned_image() {
            assert_digest_pinned_image_is_not_updated(
                "file:///Dockerfile",
                "dockerfile",
                "command-does-not-update-dockerfile-digest-pinned-imageDockerfile",
            )
            .await;
        }

        #[tokio::test]
        async fn apply_command_does_not_update_compose_digest_pinned_image() {
            assert_digest_pinned_image_is_not_updated(
                "file:///compose.yaml",
                "yaml",
                "command-does-not-update-compose-digest-pinned-image.yaml",
            )
            .await;
        }

        async fn assert_digest_pinned_image_is_not_updated(
            uri: &str,
            language_id: &str,
            fixture: &str,
        ) {
            let output = standard_session().apply_command(
        DocumentInput::new(
            uri.to_owned(),
            language_id.to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput::new(
            "ubuntu".to_owned(),
            Docker,
            r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#.to_owned(),
        )],
    ).await;
            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_vcpkg_version_constraint() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///vcpkg.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("command-updates-vcpkg-version-constraint.json"),
                        None,
                    ),
                    Some("update"),
                    Some("fmt"),
                    &[RegistryResponseInput::new(
                        "fmt".to_owned(),
                        Vcpkg,
                        r#"{"versions":[{"version":"11.1.4"},{"version":"10.1.1#1"}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "11.1.4");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_vcpkg_baseline_dependency_without_version_constraint()
         {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///vcpkg.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-does-not-update-vcpkg-unconstrained-vcpkg-dependency.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("zlib"),
                    &[RegistryResponseInput::new(
                        "zlib".to_owned(),
                        Vcpkg,
                        r#"{"versions":[{"version":"1.3.1"}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_swift_package_github_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///Package.swift".to_owned(),
                        "swift".to_owned(),
                        package_file_fixture(
                            "command-updates-swift-package-github-dependency.swift",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("swift-nio"),
                    &[RegistryResponseInput::new(
                        "apple/swift-nio".to_owned(),
                        Swift,
                        r#"[{"name":"2.66.0"},{"name":"2.65.0"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "2.66.0");
        }
        #[tokio::test]
        async fn apply_command_does_not_update_swift_local_package_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///Package.swift".to_owned(),
                        "swift".to_owned(),
                        package_file_fixture(
                            "command-does-not-update-swift-local-package-dependency.swift",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("LocalPackage"),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_zig_github_url_tag_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///build.zig.zon".to_owned(),
                        "zig".to_owned(),
                        package_file_fixture(
                            "command-updates-zig-github-url-tag-dependency.zig.zon",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("known_folders"),
                    &[RegistryResponseInput::new(
                        "ziglibs/known-folders".to_owned(),
                        Zig,
                        r#"[{"name":"0.8.0"},{"name":"0.7.0"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "0.8.0");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_zig_path_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///build.zig.zon".to_owned(),
                        "zig".to_owned(),
                        package_file_fixture("command-does-not-update-zig-path-dependency.zig.zon"),
                        None,
                    ),
                    Some("update"),
                    Some("local_dep"),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_nimble_github_url_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///demo.nimble".to_owned(),
                        "nim".to_owned(),
                        package_file_fixture("command-updates-nimble-github-url-dependency.nimble"),
                        None,
                    ),
                    Some("update"),
                    Some("pkg"),
                    &[RegistryResponseInput::new(
                        "user/pkg".to_owned(),
                        Nim,
                        r#"[{"name":"2.1.0"},{"name":"2.0.0"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "== 2.1.0");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_nimble_head_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///demo.nimble".to_owned(),
                        "nim".to_owned(),
                        package_file_fixture(
                            "command-does-not-update-nimble-head-dependency.nimble",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("foobar"),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_luarocks_rockspec_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///demo-1.0.0-1.rockspec".to_owned(),
                        "lua".to_owned(),
                        package_file_fixture(
                            "command-updates-luarocks-rockspec-dependency.0.0-1.rockspec",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("luasocket"),
                    &[RegistryResponseInput::new(
                        "luasocket".to_owned(),
                        LuaRocks,
                        r#"repository = {
   ["luasocket"] = {
      ["3.0.0-1"] = { { arch = "rockspec" } },
      ["3.1.0-1"] = { { arch = "src" } }
   }
}"#
                        .to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "== 3.1.0-1");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_luarocks_lua_runtime_dependency() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///demo-1.0.0-1.rockspec".to_owned(), "lua".to_owned(), package_file_fixture(
                "command-does-not-update-luarocks-lua-runtime-dependency.0.0-1.rockspec",
            ), None),
        Some("update"),
        Some("lua"),
        &[],
    ).await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn apply_command_updates_cpanfile_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/cpanfile".to_owned(),
                        "perl".to_owned(),
                        package_file_fixture("command-updates-cpanfile-dependencycpanfile"),
                        None,
                    ),
                    Some("update"),
                    Some("Plack"),
                    &[RegistryResponseInput::new(
                        "Plack".to_owned(),
                        Cpan,
                        r#"{"status":"latest","version":"2.0.0"}"#.to_owned(),
                    )],
                )
                .await;

            assert_single_edit(&output, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_updates_haxelib_json_dependency() {
            let session = standard_session();

            let output = session.apply_command(
        DocumentInput::new("file:///work/haxelib.json".to_owned(), "json".to_owned(), package_file_fixture("command-updates-haxelib-json-dependency.json"), None),
        Some("update"),
        Some("tink_core"),
        &[RegistryResponseInput::new("tink_core".to_owned(), Haxelib, r"<code>haxelib install tink_core 2.0.0</code><code>haxelib install tink_core 1.0.0</code>".to_owned())],
    ).await;

            assert_single_edit(&output, "2.0.0");
        }

        #[tokio::test]
        async fn apply_command_does_not_update_haxelib_latest_dependency() {
            let session = standard_session();

            let output = session
                .apply_command(
                    DocumentInput::new(
                        "file:///work/haxelib.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "command-does-not-update-haxelib-latest-dependency.json",
                        ),
                        None,
                    ),
                    Some("update"),
                    Some("tink_macro"),
                    &[RegistryResponseInput::new(
                        "tink_macro".to_owned(),
                        Haxelib,
                        r"<code>haxelib install tink_macro 2.0.0</code>".to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.edits.len(), 0);
        }
    }
}
