use crate::selection::select_dependencies;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use versionlens_update::can_sort_dependencies;
use versionlens_update::update_edits;

use versionlens_model::ManifestKind::VersionLensMultiRegistries;
use versionlens_model::{DocumentInput, ManifestKind, ProjectVersionBump};
use versionlens_parsers::{ecosystem_for_manifest, provider_name_for_manifest};
use versionlens_update::{Suggestion, SuggestionStatus};

use super::operation::OperationContext;
use super::resolution::ResolutionRequest;
use crate::VersionLensSession;
use crate::command::install_task_config_key_for_manifest;
use crate::contract::{AnalyzeDocumentOutput, RegistryResponseInput, ResolveDocumentOutput};
use crate::dependency::into_dependency_views;
use crate::presentation::status::{status_view, to_u32};
use crate::presentation::view::DiagnosticView;
use crate::project::is_project_version_dependency;
use crate::schema::schema_output;
use crate::snapshot::dependency_signature;
pub(super) struct DependencySuggestionsRequest<'a> {
    pub(super) input: DocumentInput,
    pub(super) selector: &'a str,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) project_bump: Option<ProjectVersionBump>,
    pub(super) operation: &'a OperationContext,
}

impl VersionLensSession {
    /// Whether the document is a manifest the engine supports and has on.
    pub fn document_is_supported(&self, input: &DocumentInput) -> bool {
        self.manifest_enabled(self.classify_document(input))
    }

    /// Whether the document needs no new check.
    pub fn document_is_fresh(&self, input: &DocumentInput) -> bool {
        self.document_check_delay(input)
            .is_none_or(|delay| !delay.is_zero())
    }

    /// Returns zero when checking is due, a delay until the earliest cached
    /// outcome expires, or `None` when the document needs no upstream check.
    pub fn document_check_delay(&self, input: &DocumentInput) -> Option<Duration> {
        self.synchronize_persistent_cache();
        let kind = self.classify_document(input);
        let context = self.registry_context(input, kind);
        let scope = self.document_cache_scope(&context, input);
        let dependencies = self.dependencies(input);
        let mut cache = self.suggestion_cache();
        let mut next = None;
        for dependency in &dependencies {
            if is_project_version_dependency(dependency) {
                continue;
            }
            let key = scope.suggestion_key(dependency);
            let Some((suggestion, deadline)) = cache.get_with_expiry(&key) else {
                return Some(Duration::ZERO);
            };
            let failed = matches!(
                suggestion.status,
                SuggestionStatus::Error | SuggestionStatus::RateLimited
            );
            let deadline = if failed {
                deadline
            } else {
                self.cached_update_vulnerabilities_deadline(suggestion, deadline)
            };
            next = Some(next.map_or(deadline, |previous: Instant| previous.min(deadline)));
        }
        drop(cache);
        next.map(|deadline| deadline.saturating_duration_since(Instant::now()))
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "the public API takes the document by value"
    )]
    /// Finds the dependencies of a document without a registry lookup.
    pub fn analyze_document(&self, input: DocumentInput) -> AnalyzeDocumentOutput {
        self.analyze_document_with_responses(&input, &[])
    }

    pub(crate) fn analyze_document_with_responses(
        &self,
        input: &DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> AnalyzeDocumentOutput {
        self.synchronize_persistent_cache();
        let manifest_kind = self.classify_document(input);
        let is_supported_manifest = self.manifest_enabled(manifest_kind);
        let active_provider_name = is_supported_manifest
            .then(|| provider_name_for_manifest(manifest_kind))
            .flatten()
            .map(ToOwned::to_owned);
        if manifest_kind == VersionLensMultiRegistries {
            return schema_output(input);
        }

        let dependencies = self.dependencies(input);
        let context = self.registry_context(input, manifest_kind);
        let scope = self.document_cache_scope(&context, input);
        let suggestions = dependencies
            .iter()
            .map(|dependency| self.cached_suggestion(dependency, &scope))
            .collect::<Vec<_>>();
        let code_lenses = dependencies
            .iter()
            .flat_map(|dependency| self.code_lenses_for_dependency(dependency, &scope))
            .collect();
        let mut diagnostic_ranges: HashSet<(u32, u32, u32, u32)> = HashSet::default();
        let mut diagnostics: Vec<DiagnosticView> = dependencies
            .iter()
            .filter(|dependency| {
                let range = dependency.requirement_range;
                diagnostic_ranges.insert((
                    range.start.line,
                    range.start.character,
                    range.end.line,
                    range.end.character,
                ))
            })
            .flat_map(|dependency| self.diagnostics_for_dependency(dependency, responses))
            .collect();

        let status = status_view(
            dependencies.len(),
            &diagnostics,
            &suggestions,
            self.config.show_suggestion_stats,
        );
        let mut update_ranges: HashSet<(u32, u32, u32, u32)> = HashSet::default();
        diagnostics.extend(
            suggestions
                .iter()
                .flatten()
                .filter_map(|suggestion| self.update_diagnostic(suggestion))
                .filter(|diagnostic| {
                    let range = diagnostic.range;
                    update_ranges.insert((
                        range.start.line,
                        range.start.character,
                        range.end.line,
                        range.end.character,
                    ))
                }),
        );
        let install_task_config_key = (!dependencies.is_empty())
            .then(|| install_task_config_key_for_manifest(manifest_kind))
            .flatten();
        let dependency_signature = dependency_signature(&dependencies);
        let can_sort_dependencies = can_sort_dependencies(&dependencies);
        let dependency_views = into_dependency_views(dependencies);

        AnalyzeDocumentOutput {
            can_sort_dependencies,
            is_supported_manifest,
            active_provider_name,
            install_task_config_key,
            dependency_signature,
            dependencies: dependency_views,
            code_lenses,
            diagnostics,
            status,
        }
    }

    fn manifest_enabled(&self, manifest_kind: ManifestKind) -> bool {
        match ecosystem_for_manifest(manifest_kind) {
            Some(ecosystem) => self.provider_enabled_for_manifest(manifest_kind, ecosystem),
            None => manifest_kind == VersionLensMultiRegistries,
        }
    }

    pub(crate) async fn resolve_document(&self, input: DocumentInput) -> ResolveDocumentOutput {
        self.resolve_document_with_responses(input, &[]).await
    }

    /// Resolves the versions of a document, with registry responses the editor already has.
    pub async fn resolve_document_with_responses(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        let operation = self.operation_context();
        let manifest_kind = self.classify_document(&input);
        let plan_input = input.clone();
        let suggestions = self
            .resolve_suggestions(input, responses, None, &operation)
            .await;
        let edits = update_edits(&suggestions);
        let authorization_required_count = Self::authorization_required_count(&suggestions);
        let vulnerable_update_count = self
            .vulnerable_update_count(&suggestions, responses, Some(manifest_kind), &operation)
            .await;
        let authorization_required_requests = operation.take_authorization_requests();
        let authorization_required_count =
            authorization_required_count.max(to_u32(authorization_required_requests.len()));
        let parts = super::resolve_output_parts_with_plan(
            &plan_input,
            &edits,
            authorization_required_count,
            authorization_required_requests,
            vulnerable_update_count,
        );
        if !operation.can_publish() {
            return super::cancelled_resolution(suggestions);
        }
        super::finish_resolve_output(suggestions, parts)
    }

    pub(crate) async fn resolve_suggestions(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
        project_bump: Option<ProjectVersionBump>,
        operation: &OperationContext,
    ) -> Vec<Suggestion> {
        let manifest_kind = self.classify_document(&input);
        let context = self.registry_context(&input, manifest_kind);
        let suggestions = self
            .resolve_dependencies(ResolutionRequest {
                input: &input,
                dependencies: if project_bump.is_some() {
                    self.command_dependencies(&input)
                } else {
                    self.dependencies(&input)
                },
                document_uri: &input.uri,
                responses,
                project_bump,
                context: &context,
                operation,
            })
            .await;
        if project_bump.is_none() {
            self.cache_resolved_suggestions(
                &suggestions,
                context.manifest_kind(),
                operation,
                &self.document_cache_scope(&context, &input),
            );
        }
        suggestions
    }

    pub(super) async fn resolve_dependency_suggestions(
        &self,
        request: DependencySuggestionsRequest<'_>,
    ) -> Vec<Suggestion> {
        let DependencySuggestionsRequest {
            input,
            selector,
            responses,
            project_bump,
            operation,
        } = request;
        let manifest_kind = self.classify_document(&input);
        let context = self.registry_context(&input, manifest_kind);
        let dependencies = if project_bump.is_some() {
            self.command_dependencies(&input)
        } else {
            self.dependencies(&input)
        };
        let dependencies = select_dependencies(dependencies, selector);
        self.resolve_dependencies(ResolutionRequest {
            input: &input,
            dependencies,
            document_uri: &input.uri,
            responses,
            project_bump,
            context: &context,
            operation,
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::ProviderSettings;
    use crate::{
        DependencyPropertyConfig, RegistryResponseInput, SessionConfig, VersionLensSession,
    };
    use serde_json::to_value;

    use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

    use versionlens_model::Ecosystem::*;
    use versionlens_model::ManifestKind::NpmPackageJson;

    #[test]
    fn reports_sort_capability_only_for_supported_documents() {
        let session = standard_session(false);
        let package_json_catalog_session = session_with_dependency_properties(
            false,
            Npm,
            Some(NpmPackageJson),
            &["workspaces.catalogs.*"],
        );
        let input = |uri: &str, language_id: &str, text: &str| {
            DocumentInput::new(
                uri.to_owned(),
                language_id.to_owned(),
                text.to_owned(),
                None,
            )
        };

        let requirements = session.analyze_document(input(
            "file:///requirements.txt",
            "pip-requirements",
            package_file_fixture("requirements-unsorted.txt").as_str(),
        ));
        let package_json = session.analyze_document(input(
            "file:///package.json",
            "json",
            package_file_fixture("single-line.json").as_str(),
        ));
        let multiline_package_json = session.analyze_document(input(
            "file:///package.json",
            "json",
            package_file_fixture("dependencies-unsorted.json").as_str(),
        ));
        let package_json_with_metadata = session.analyze_document(input(
            "file:///package.json",
            "json",
            package_file_fixture("with-package-tool.json").as_str(),
        ));
        let pubspec = session.analyze_document(input(
            "file:///pubspec.yaml",
            "yaml",
            package_file_fixture("pubspec-unsorted.yaml").as_str(),
        ));
        let composer = session.analyze_document(input(
            "file:///composer.json",
            "json",
            package_file_fixture("composer-unsorted.json").as_str(),
        ));
        let pnpm_workspace = session.analyze_document(input(
            "file:///pnpm-workspace.yaml",
            "yaml",
            package_file_fixture("workspace-catalogs-unsorted.yaml").as_str(),
        ));
        let package_json_workspace_catalogs = package_json_catalog_session.analyze_document(input(
            "file:///package.json",
            "json",
            package_file_fixture("workspace-catalogs-unsorted.json").as_str(),
        ));
        let maven = session.analyze_document(input(
            "file:///pom.xml",
            "xml",
            crate::test_support::fixture(
                "tests/fixtures/session/commands/sort",
                "dependencies-unsorted.xml",
            )
            .as_str(),
        ));
        let dotnet = session.analyze_document(input(
            "file:///app.csproj",
            "xml",
            package_file_fixture("app-unsorted.csproj").as_str(),
        ));
        let go_mod = session.analyze_document(input(
            "file:///go.mod",
            "go.mod",
            package_file_fixture("go-unsorted.mod").as_str(),
        ));
        let empty_package_json = session.analyze_document(input(
            "file:///package.json",
            "json",
            package_file_fixture("empty-package.json").as_str(),
        ));
        let unsupported =
            session.analyze_document(input("file:///notes.txt", "plaintext", "hello"));

        assert!(requirements.can_sort_dependencies);
        assert!(pubspec.can_sort_dependencies);
        assert!(multiline_package_json.can_sort_dependencies);
        assert!(package_json_with_metadata.can_sort_dependencies);
        assert!(composer.can_sort_dependencies);
        assert!(pnpm_workspace.can_sort_dependencies);
        assert!(package_json_workspace_catalogs.can_sort_dependencies);
        assert!(maven.can_sort_dependencies);
        assert!(dotnet.can_sort_dependencies);
        assert!(go_mod.can_sort_dependencies);
        assert!(!package_json.can_sort_dependencies);
        assert!(empty_package_json.is_supported_manifest);
        assert!(!unsupported.is_supported_manifest);
    }

    #[test]
    fn reports_sort_capability_for_deno_scoped_imports() {
        let session = session_with_dependency_properties(false, Deno, None, &["scopes"]);

        let output = session.analyze_document(DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("scopes-unsorted.json"),
            None,
        ));

        assert!(output.can_sort_dependencies);
    }

    #[test]
    fn reports_sort_capability_for_gemfile_dependencies() {
        let session = standard_session(false);

        let output = session.analyze_document(DocumentInput::new(
            "file:///Gemfile".to_owned(),
            "ruby".to_owned(),
            package_file_fixture("Gemfile-unsorted"),
            None,
        ));

        assert!(output.can_sort_dependencies);
    }

    #[test]
    fn analyze_document_reports_active_provider_name_for_supported_manifests() {
        let session = standard_session(false);

        let npm = session.analyze_document(package_json_input(
            package_file_fixture("empty-package.json").as_str(),
        ));
        let package_json5 = session.analyze_document(DocumentInput::new(
            "file:///package.json5".to_owned(),
            "json5".to_owned(),
            package_file_fixture("provider.json5"),
            None,
        ));
        let package_yaml = session.analyze_document(DocumentInput::new(
            "file:///package.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("provider.yaml"),
            None,
        ));
        let deno_import_map = session.analyze_document(DocumentInput::new(
            "file:///import_map.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("import-map-provider.json"),
            None,
        ));
        let jsr = session.analyze_document(DocumentInput::new(
            "file:///jsr.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("jsr-provider.json"),
            None,
        ));
        let pnpm = session.analyze_document(DocumentInput::new(
            "file:///pnpm-workspace.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("workspace-provider.yaml"),
            None,
        ));
        let golang = session.analyze_document(DocumentInput::new(
            "file:///go.mod".to_owned(),
            "go.mod".to_owned(),
            package_file_fixture("go-provider.mod"),
            None,
        ));
        let pypi = session.analyze_document(DocumentInput::new(
            "file:///requirements.txt".to_owned(),
            "pip-requirements".to_owned(),
            package_file_fixture("requirements-provider.txt"),
            None,
        ));
        let ruby_gemspec = session.analyze_document(DocumentInput::new(
            "file:///example.gemspec".to_owned(),
            "ruby".to_owned(),
            package_file_fixture("example-provider.gemspec"),
            None,
        ));
        let unsupported = session.analyze_document(DocumentInput::new(
            "file:///notes.txt".to_owned(),
            "plaintext".to_owned(),
            "hello".to_owned(),
            None,
        ));

        assert_eq!(npm.active_provider_name, Some("npm".to_owned()));
        assert_eq!(package_json5.active_provider_name, Some("npm".to_owned()));
        assert_eq!(package_yaml.active_provider_name, Some("npm".to_owned()));
        assert_eq!(
            deno_import_map.active_provider_name,
            Some("deno".to_owned())
        );
        assert_eq!(jsr.active_provider_name, Some("deno".to_owned()));
        assert_eq!(pnpm.active_provider_name, Some("pnpm".to_owned()));
        assert_eq!(golang.active_provider_name, Some("golang".to_owned()));
        assert_eq!(pypi.active_provider_name, Some("pypi".to_owned()));
        assert_eq!(ruby_gemspec.active_provider_name, Some("ruby".to_owned()));
        let terraform = session.analyze_document(DocumentInput::new(
            "file:///main.tf".to_owned(),
            "terraform".to_owned(),
            package_file_fixture("terraform-provider.tf"),
            None,
        ));
        let helm = session.analyze_document(DocumentInput::new(
            "file:///Chart.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("Chart-provider.yaml"),
            None,
        ));
        assert_eq!(terraform.active_provider_name, Some("terraform".to_owned()));
        assert_eq!(helm.active_provider_name, Some("helm".to_owned()));
        assert_eq!(unsupported.active_provider_name, None);
    }

    #[test]
    fn analyze_document_serializes_dependencies_as_vscode_payloads() {
        let session = standard_session(false);

        let output = session.analyze_document(DocumentInput::new(
            "file:///go.mod".to_owned(),
            "go.mod".to_owned(),
            package_file_fixture("go-single-require.mod"),
            None,
        ));
        let value = to_value(output).unwrap();

        assert_eq!(value["dependencies"][0]["name"], "example.test/pkg");
        assert_eq!(value["dependencies"][0]["ecosystem"], "golang");
    }

    #[tokio::test]
    async fn resolve_document_serializes_suggestions_as_vscode_payloads() {
        let session = standard_session(false);

        let output = session
            .resolve_document_with_responses(
                DocumentInput::new(
                    "file:///go.mod".to_owned(),
                    "go.mod".to_owned(),
                    package_file_fixture("go-single-require.mod"),
                    None,
                ),
                &[RegistryResponseInput::new(
                    "example.test/pkg".to_owned(),
                    Go,
                    "v1.1.0\n".to_owned(),
                )],
            )
            .await;
        let value = to_value(output).unwrap();

        assert_eq!(
            value["suggestions"][0]["dependency"]["name"],
            "example.test/pkg"
        );
        assert_eq!(value["suggestions"][0]["dependency"]["ecosystem"], "golang");
        assert_eq!(value["suggestions"][0]["status"], "updateAvailable");
    }

    #[tokio::test]
    async fn deno_non_jsr_npm_imports_produce_no_suggestions_like_upstream() {
        let session = standard_session(false);
        let input = DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("remote-import.json"),
            None,
        );

        let output = session
            .resolve_document_with_responses(input.clone(), &[])
            .await;
        let analysis = session.analyze_document(input);

        assert_eq!(analysis.dependencies.len(), 1);
        assert_eq!(output.suggestions.len(), 0);
        assert_eq!(analysis.code_lenses.len(), 0);
        assert_eq!(output.edits.len(), 0);
    }

    #[tokio::test]
    async fn analyze_document_uses_cached_latest_for_diagnostics() {
        let mut config = standard_config(false);
        let session = crate::version_lens_session(config.clone());
        let input = package_json_input(package_file_fixture("single-line.json").as_str());

        session
            .resolve_document_with_responses(input.clone(), &[registry_response()])
            .await;

        let output = session.analyze_document(input.clone());

        let codes = output
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_deref(), diagnostic.message.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(codes, [(Some(crate::UPDATE_AVAILABLE), "1.0.0 → 1.1.0")]);
        assert_eq!(output.status.dependency_count, 1);
        assert_eq!(output.status.update_count, 1);
        assert_eq!(output.status.vulnerability_count, 0);
        assert!(output.status.visible);
        assert_eq!(output.status.text, "$(versions) 1/1");
        assert!(!output.can_sort_dependencies);
        assert_eq!(
            output.install_task_config_key,
            Some("npm.onSaveChanges".to_owned())
        );
        assert_eq!(
            output.dependency_signature,
            r#"["npm","left-pad","dependencies","1.0.0",null,null,null]"#
        );

        config.show_suggestion_stats = true;
        let session = crate::version_lens_session(config);
        session
            .resolve_document_with_responses(input.clone(), &[registry_response()])
            .await;

        assert_eq!(
            session.analyze_document(input).status.text,
            "$(versions) 1/1 updates, 0 vulnerabilities, 0 errors, 0 no matches"
        );
    }

    #[tokio::test]
    async fn analyze_document_reports_cached_errors_and_no_matches_in_status() {
        let session = standard_session(true);
        let input = package_json_input(package_file_fixture("missing-and-errored.json").as_str());

        let output = crate::test_support::analyze_with_responses(
            &session,
            &input,
            &[
                RegistryResponseInput::new(
                    "missing-package".to_owned(),
                    Npm,
                    r#"{"versions":{}}"#.to_owned(),
                ),
                RegistryResponseInput::new(
                    "errored-package".to_owned(),
                    Npm,
                    r#"{"status":"E404"}"#.to_owned(),
                ),
            ],
        )
        .await;

        assert_eq!(output.status.update_count, 0);
        assert_eq!(output.status.error_count, 1);
        assert_eq!(output.status.no_match_count, 1);
        assert_eq!(
            output.status.text,
            "$(versions) 0/2 updates, 0 vulnerabilities, 1 errors, 1 no matches"
        );
    }

    #[test]
    fn analyze_document_reports_when_sort_is_unavailable() {
        let session = standard_session(false);

        let output = session.analyze_document(package_json_input(
            package_file_fixture("empty-package.json").as_str(),
        ));

        assert!(!output.can_sort_dependencies);
        assert_eq!(output.install_task_config_key, None);
        assert_eq!(output.dependency_signature, "");
    }

    #[test]
    fn analyze_document_uses_manifest_for_install_task_key() {
        let session = standard_session(false);

        let output = session.analyze_document(DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("npm-import.json"),
            None,
        ));

        assert_eq!(
            output.install_task_config_key,
            Some("deno.onSaveChanges".to_owned())
        );
    }

    #[test]
    fn analyze_document_keeps_package_json_install_task_on_npm_provider() {
        let session = standard_session(false);

        let output = session.analyze_document(DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("pnpm-tool.json"),
            None,
        ));

        assert_eq!(
            output.install_task_config_key,
            Some("npm.onSaveChanges".to_owned())
        );
    }

    #[test]
    fn analyze_document_does_not_offer_install_task_for_pnpm_yaml() {
        let session = standard_session(false);

        let output = session.analyze_document(DocumentInput::new(
            "file:///pnpm-workspace.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("workspace-catalog.yaml"),
            None,
        ));

        assert_eq!(output.dependencies.len(), 1);
        assert_eq!(output.install_task_config_key, None);
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/session/documents", name)
    }

    fn standard_session(show_suggestion_stats: bool) -> VersionLensSession {
        crate::version_lens_session(standard_config(show_suggestion_stats))
    }

    fn standard_config(show_suggestion_stats: bool) -> SessionConfig {
        SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings::default(),
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: true,
            show_suggestion_stats,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        }
    }

    fn session_with_dependency_properties(
        show_suggestion_stats: bool,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
        properties: &[&str],
    ) -> VersionLensSession {
        let mut config = standard_config(show_suggestion_stats);
        config.providers.dependency_properties = vec![DependencyPropertyConfig {
            ecosystem,
            manifest_kind,
            properties: properties
                .iter()
                .map(|property| (*property).to_owned())
                .collect(),
        }];
        crate::version_lens_session(config)
    }

    fn package_json_input(text: &str) -> DocumentInput {
        DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            text.to_owned(),
            None,
        )
    }

    fn registry_response() -> RegistryResponseInput {
        RegistryResponseInput::new(
            "left-pad".to_owned(),
            Npm,
            r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        )
    }

    #[tokio::test]
    async fn document_freshness_tracks_version_and_advisory_expiry() {
        let session = standard_session(false);
        let input = package_json_input(r#"{"dependencies":{"example":"1.0.0"}}"#);
        let responses = [RegistryResponseInput::new(
            "example",
            Npm,
            r#"{"dist-tags":{"latest":"2.0.0"}}"#,
        )];
        assert!(!session.document_is_fresh(&input));
        session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        assert!(session.document_is_fresh(&input));
        session.vulnerability_cache().clear();
        assert!(!session.document_is_fresh(&input));
        session
            .resolve_document_with_responses(input.clone(), &responses)
            .await;
        assert!(session.document_is_fresh(&input));
        let dependency = session.dependencies(&input).remove(0);
        let context = session.registry_context(&input, session.classify_document(&input));
        let scope = session.document_cache_scope(&context, &input);
        let cached = session
            .cached_resolved_suggestion(&dependency, &scope)
            .unwrap();
        session.suggestion_cache().insert_with_ttl(
            scope.suggestion_key(&dependency),
            cached,
            std::time::Duration::ZERO,
        );
        assert!(!session.document_is_fresh(&input));
    }

    #[tokio::test]
    async fn freshness_does_not_transfer_to_a_changed_requirement_or_another_document() {
        let session = standard_session(false);
        let input = package_json_input(r#"{"dependencies":{"example":"1.0.0"}}"#);
        session
            .resolve_document_with_responses(
                input.clone(),
                &[RegistryResponseInput::new(
                    "example",
                    Npm,
                    r#"{"dist-tags":{"latest":"2.0.0"}}"#,
                )],
            )
            .await;
        assert!(session.document_is_fresh(&input));
        let mut changed = input.clone();
        changed.text = changed.text.replace("1.0.0", "1.1.0");
        assert!(!session.document_is_fresh(&changed));
        changed = input.clone();
        changed.uri = "file:///another/package.json".to_owned();
        assert!(!session.document_is_fresh(&changed));
        session.clear_cache();
        assert!(!session.document_is_fresh(&input));
    }

    #[tokio::test]
    async fn document_check_deadline_uses_the_earliest_advisory_expiry() {
        use std::time::Duration;
        let session = standard_session(false);
        let input = package_json_input(r#"{"dependencies":{"example":"1.0.0"}}"#);
        assert_eq!(session.document_check_delay(&input), Some(Duration::ZERO));
        session
            .resolve_document_with_responses(
                input.clone(),
                &[RegistryResponseInput::new(
                    "example",
                    Npm,
                    r#"{"dist-tags":{"latest":"2.0.0"}}"#,
                )],
            )
            .await;
        let delay = session.document_check_delay(&input).unwrap();
        assert!(delay > Duration::from_secs(30));
        assert!(delay <= Duration::from_secs(300));
        let dependency = session.dependencies(&input).remove(0);
        session.vulnerability_cache().insert_with_ttl(
            crate::cache::vulnerability_cache_key(&dependency),
            crate::vulnerability::VulnerabilityCheck::Checked(vec![]),
            Duration::from_secs(7),
        );
        let delay = session.document_check_delay(&input).unwrap();
        assert!(!delay.is_zero());
        assert!(delay <= Duration::from_secs(7));
        session.vulnerability_cache().clear();
        assert_eq!(session.document_check_delay(&input), Some(Duration::ZERO));
    }

    #[tokio::test]
    async fn failed_checks_wait_for_retry_and_empty_documents_need_no_timer() {
        use std::time::Duration;
        let session = standard_session(false);
        let input = package_json_input(r#"{"dependencies":{"example":"1.0.0"}}"#);
        let output = session
            .resolve_document_with_responses(
                input.clone(),
                &[RegistryResponseInput::new(
                    "example",
                    Npm,
                    r#"{"status":"E404"}"#,
                )],
            )
            .await;
        assert_eq!(output.suggestions[0].status, "error");
        session.vulnerability_cache().clear();
        let delay = session.document_check_delay(&input).unwrap();
        assert!(!delay.is_zero());
        assert!(delay <= Duration::from_secs(30));
        assert_eq!(
            session.document_check_delay(&package_json_input("{}")),
            None
        );
        session.clear_cache();
        assert_eq!(session.document_check_delay(&input), Some(Duration::ZERO));
    }

    mod updates {
        use versionlens_model::Ecosystem::Npm;

        use super::{package_json_input, standard_config};
        use crate::{AnalyzeDocumentOutput, RegistryResponseInput, SessionConfig, Severity};

        const ASTRO: &str =
            r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#;

        async fn analyze(
            config: SessionConfig,
            manifest: &str,
            package: &str,
            body: &str,
        ) -> AnalyzeDocumentOutput {
            let session = crate::version_lens_session(config);
            let input = package_json_input(manifest);
            let responses = [RegistryResponseInput::new(package, Npm, body)];
            crate::test_support::analyze_with_responses(&session, &input, &responses).await
        }

        fn update_diagnostics(output: &AnalyzeDocumentOutput) -> Vec<&crate::DiagnosticView> {
            output
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code.as_deref() == Some(crate::UPDATE_AVAILABLE))
                .collect()
        }

        #[tokio::test]
        async fn a_range_that_excludes_latest_gets_an_update_diagnostic_on_its_requirement() {
            let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
            let output = analyze(standard_config(false), manifest, "astro", ASTRO).await;

            let diagnostics = update_diagnostics(&output);
            assert_eq!(diagnostics.len(), 1, "{:?}", output.diagnostics);
            let diagnostic = diagnostics[0];
            assert_eq!(diagnostic.message, "^7.2.2 → ^8.0.1");
            assert_eq!(diagnostic.severity, Severity::Information);
            assert_eq!(diagnostic.source.as_deref(), Some("VersionLens"));
            let requirement = crate::version_lens_session(standard_config(false))
                .dependencies(&package_json_input(manifest))
                .remove(0)
                .requirement_range;
            assert_eq!(diagnostic.range, requirement);
        }

        #[tokio::test]
        async fn a_range_that_allows_latest_keeps_its_lens_and_gets_no_diagnostic() {
            let output = analyze(
                standard_config(false),
                r#"{"devDependencies":{"@biomejs/biome":"^2.5.8"}}"#,
                "@biomejs/biome",
                r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
            )
            .await;

            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
            assert_ne!(output.code_lenses.len(), 0);
        }

        #[tokio::test]
        async fn the_configured_severity_is_used_and_off_publishes_none() {
            let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
            let mut config = standard_config(false);
            config.update_severity = Some(Severity::Hint);
            let output = analyze(config.clone(), manifest, "astro", ASTRO).await;
            assert_eq!(update_diagnostics(&output)[0].severity, Severity::Hint);

            config.update_severity = None;
            let output = analyze(config, manifest, "astro", ASTRO).await;
            assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
        }

        #[tokio::test]
        async fn a_vulnerable_outdated_dependency_gets_both_diagnostics() {
            let session = crate::version_lens_session(standard_config(false));
            let input = package_json_input(r#"{"dependencies":{"left-pad":"1.0.0"}}"#);
            session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "left-pad",
                        Npm,
                        r#"{"dist-tags":{"latest":"1.1.0"}}"#,
                    )],
                )
                .await;

            let output = session.analyze_document_with_responses(
                &input,
                &[crate::test_support::npm_vulnerability_range_response(
                    "left-pad",
                    "OSV-1",
                    "prototype issue",
                    "1.1.0",
                    None,
                )],
            );

            let codes = output
                .diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.code.as_deref(), diagnostic.severity))
                .collect::<Vec<_>>();
            assert_eq!(
                codes,
                [
                    (Some("OSV-1"), Severity::Error),
                    (Some(crate::UPDATE_AVAILABLE), Severity::Information),
                ]
            );
            assert_eq!(output.status.vulnerability_count, 1);
        }

        #[tokio::test]
        async fn a_floor_bump_to_a_new_minor_gets_a_diagnostic_and_a_patch_bump_does_not() {
            let body = r#"{"dist-tags":{"latest":"7.3.5"},"versions":{"7.2.2":{},"7.3.5":{}}}"#;
            for (requirement, expected) in [
                ("^7.2.2", Some("^7.2.2 → ^7.3.5")),
                ("~7.2.2", Some("~7.2.2 → ~7.3.5")),
                ("^7.3.0", None),
                ("^7.3.5", None),
            ] {
                let manifest = format!(r#"{{"dependencies":{{"astro":"{requirement}"}}}}"#);
                let output = analyze(standard_config(false), &manifest, "astro", body).await;
                let messages = update_diagnostics(&output)
                    .iter()
                    .map(|diagnostic| diagnostic.message.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(messages, Vec::from_iter(expected), "{requirement}");
            }
        }
    }

    mod actions {
        use versionlens_model::Ecosystem::Npm;
        use versionlens_model::{Position, Range};

        use super::{package_json_input, standard_config};
        use crate::{RegistryResponseInput, VersionLensSession};

        const ASTRO: &str =
            r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#;

        async fn resolved(manifest: &str, package: &str, body: &str) -> VersionLensSession {
            let session = crate::version_lens_session(standard_config(false));
            session
                .resolve_document_with_responses(
                    package_json_input(manifest),
                    &[RegistryResponseInput::new(package, Npm, body)],
                )
                .await;
            session
        }

        const fn at(character: u32) -> Range {
            let position = Position { line: 0, character };
            Range {
                start: position,
                end: position,
            }
        }

        fn summary(actions: &[crate::BumpActionView]) -> Vec<(&str, &str, bool, bool)> {
            actions
                .iter()
                .map(|action| {
                    (
                        action.title.as_str(),
                        action.edit.new_text.as_str(),
                        action.preferred,
                        action.diagnostic.is_some(),
                    )
                })
                .collect()
        }

        #[tokio::test]
        async fn each_offered_bump_is_an_action_with_latest_preferred() {
            let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
            let session = resolved(manifest, "astro", ASTRO).await;

            let actions = session.bump_actions(&package_json_input(manifest), at(28));

            assert_eq!(
                summary(&actions),
                [
                    ("Bump to ^7.3.5", "^7.3.5", false, true),
                    ("Bump to ^8.0.1", "^8.0.1", true, true),
                ]
            );
            let requirement = session
                .dependencies(&package_json_input(manifest))
                .remove(0)
                .requirement_range;
            assert!(
                actions
                    .iter()
                    .all(|action| action.edit.range == requirement)
            );
        }

        #[tokio::test]
        async fn a_patch_floor_bump_is_offered_without_a_diagnostic() {
            let manifest = r#"{"devDependencies":{"@biomejs/biome":"^2.5.8"}}"#;
            let session = resolved(
                manifest,
                "@biomejs/biome",
                r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
            )
            .await;

            let actions = session.bump_actions(&package_json_input(manifest), at(40));

            assert_eq!(
                summary(&actions),
                [("Bump to ^2.5.14", "^2.5.14", true, false)]
            );
        }

        #[tokio::test]
        async fn ranges_outside_a_requirement_and_unresolved_documents_get_no_actions() {
            let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
            let session = resolved(manifest, "astro", ASTRO).await;
            assert_eq!(
                session
                    .bump_actions(&package_json_input(manifest), at(3))
                    .len(),
                0
            );

            let unresolved = crate::version_lens_session(standard_config(false));
            assert_eq!(
                unresolved
                    .bump_actions(&package_json_input(manifest), at(28))
                    .len(),
                0
            );
        }
    }
}
