use std::collections::HashSet;

use versionlens_parsers::{DocumentInput, ManifestKind, ecosystem_for_manifest};
use versionlens_suggestions::Suggestion;
use versionlens_versions::ProjectVersionBump;
use versionlens_vscode_model::DiagnosticPayload;

use crate::VersionLensSession;
use crate::command::install_task_config_key_for_manifest;
use crate::dependency::into_dependency_payloads;
use crate::model::{AnalyzeDocumentOutput, RegistryResponseInput, ResolveDocumentOutput};
use crate::registry::RegistryContext;
use crate::schema::schema_output;
use crate::snapshot::dependency_signature;
use crate::status::{status_payload, to_u32};
use crate::suggestion::into_suggestion_payloads;

impl VersionLensSession {
    pub fn analyze_document(&self, input: DocumentInput) -> AnalyzeDocumentOutput {
        self.analyze_document_with_responses(input, &[])
    }

    pub fn analyze_document_with_responses(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> AnalyzeDocumentOutput {
        let manifest_kind = self.classify_document(&input);
        let is_supported_manifest = self.manifest_enabled(manifest_kind);
        let active_provider_name = is_supported_manifest
            .then(|| active_provider_name_for_manifest(manifest_kind))
            .flatten()
            .map(str::to_owned);
        if manifest_kind == ManifestKind::VersionLensMultiRegistries {
            return schema_output(&input);
        }

        let dependencies = self.dependencies(&input);
        let suggestions = dependencies
            .iter()
            .map(|dependency| self.cached_suggestion(dependency))
            .collect::<Vec<_>>();
        let code_lenses = dependencies
            .iter()
            .flat_map(|dependency| self.code_lenses_for_dependency(dependency))
            .collect();
        let mut diagnostic_ranges = HashSet::new();
        let diagnostics: Vec<DiagnosticPayload> = dependencies
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

        let status = status_payload(
            dependencies.len(),
            &diagnostics,
            &suggestions,
            self.config.show_suggestion_stats,
        );
        let install_task_config_key = (!dependencies.is_empty())
            .then(|| install_task_config_key_for_manifest(manifest_kind))
            .flatten();
        let dependency_signature = dependency_signature(&dependencies);
        let can_sort_dependencies = versionlens_edits::can_sort_dependencies(&dependencies);
        let dependency_payloads = into_dependency_payloads(dependencies);

        AnalyzeDocumentOutput {
            can_sort_dependencies,
            is_supported_manifest,
            active_provider_name,
            install_task_config_key,
            dependency_signature,
            dependencies: dependency_payloads,
            code_lenses,
            diagnostics,
            status,
        }
    }

    fn manifest_enabled(&self, manifest_kind: ManifestKind) -> bool {
        match ecosystem_for_manifest(manifest_kind) {
            Some(ecosystem) => self.provider_enabled_for_manifest(manifest_kind, ecosystem),
            None => manifest_kind == ManifestKind::VersionLensMultiRegistries,
        }
    }

    pub fn resolve_document(&self, input: DocumentInput) -> ResolveDocumentOutput {
        self.resolve_document_with_responses(input, &[])
    }

    pub fn resolve_document_with_responses(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        self.clear_authorization_requests();
        let manifest_kind = self.classify_document(&input);
        let suggestions = self.resolve_suggestions(input, responses, None);
        let edits = versionlens_edits::update_edits(&suggestions);
        let authorization_required_count = Self::authorization_required_count(&suggestions);
        let vulnerable_update_count =
            self.vulnerable_update_count(&suggestions, responses, Some(manifest_kind));
        let authorization_required_requests = self.take_authorization_requests();
        let authorization_required_count =
            authorization_required_count.max(to_u32(authorization_required_requests.len()));
        let suggestion_payloads = into_suggestion_payloads(suggestions);
        ResolveDocumentOutput {
            suggestions: suggestion_payloads,
            edits,
            authorization_required_count,
            authorization_required_requests,
            vulnerable_update_count,
            vulnerable_update_package: None,
            vulnerable_update_version: None,
        }
    }

    pub(crate) fn resolve_suggestions(
        &self,
        input: DocumentInput,
        responses: &[RegistryResponseInput],
        project_bump: Option<ProjectVersionBump>,
    ) -> Vec<Suggestion> {
        let manifest_kind = self.classify_document(&input);
        let context = RegistryContext::from_document_kind(&input, manifest_kind);
        let suggestions = self.resolve_dependencies(
            self.dependencies(&input),
            &input.uri,
            responses,
            project_bump,
            &context,
        );
        if project_bump.is_none() {
            self.cache_resolved_suggestions(&suggestions, context.manifest_kind());
        }
        suggestions
    }

    pub(crate) fn resolve_dependency_suggestions(
        &self,
        input: DocumentInput,
        selector: &str,
        responses: &[RegistryResponseInput],
        project_bump: Option<ProjectVersionBump>,
    ) -> Vec<Suggestion> {
        let manifest_kind = self.classify_document(&input);
        let context = RegistryContext::from_document_kind(&input, manifest_kind);
        let dependencies = self
            .dependencies(&input)
            .into_iter()
            .filter(|dependency| crate::selection::matches_dependency(dependency, selector))
            .collect();
        self.resolve_dependencies(dependencies, &input.uri, responses, project_bump, &context)
    }
}

#[cfg(test)]
mod tests;

fn active_provider_name_for_manifest(manifest_kind: ManifestKind) -> Option<&'static str> {
    match manifest_kind {
        ManifestKind::CargoToml => Some("cargo"),
        ManifestKind::ComposerJson => Some("composer"),
        ManifestKind::DenoJson => Some("deno"),
        ManifestKind::DotnetProjectJson | ManifestKind::DotnetXml => Some("dotnet"),
        ManifestKind::DockerComposeYaml | ManifestKind::Dockerfile => Some("docker"),
        ManifestKind::DubJson => Some("dub"),
        ManifestKind::Gemfile => Some("ruby"),
        ManifestKind::GoMod => Some("golang"),
        ManifestKind::MavenPomXml => Some("maven"),
        ManifestKind::NpmPackageJson => Some("npm"),
        ManifestKind::PnpmYaml => Some("pnpm"),
        ManifestKind::PubspecYaml => Some("pub"),
        ManifestKind::PythonPipfile
        | ManifestKind::PythonPyprojectToml
        | ManifestKind::PythonRequirementsTxt => Some("pypi"),
        ManifestKind::Unknown | ManifestKind::VersionLensMultiRegistries => None,
    }
}
