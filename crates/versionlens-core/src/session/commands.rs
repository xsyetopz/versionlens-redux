use std::collections::HashSet;
use std::panic::resume_unwind;
use std::sync::Arc;

use versionlens_edits::bulk_update_edits;
use versionlens_edits::sort_dependency_edits;
use versionlens_edits::update_edits;
use versionlens_model::{Dependency, DocumentInput, ManifestKind};
use versionlens_suggestions::Suggestion;
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Directory as StatusDirectory,
    DirectoryNotFound as StatusDirectoryNotFound, Error as StatusError, Fixed as StatusFixed,
    NotSupported as StatusNotSupported, Unresolved as StatusUnresolved,
    UpdateAvailable as StatusUpdateAvailable,
};

use super::documents::DependencySuggestionsRequest;
use super::operation::OperationContext;
use crate::VersionLensSession;
use crate::cache::vulnerability_cache_key;
use crate::command::{filter_update_command, project_version_bump};
use crate::contract::{RegistryResponseInput, ResolveDocumentOutput};
use crate::project::is_project_version_dependency;
use crate::selection;
use crate::status::to_u32;

pub struct ApplyCommandRequest<'a> {
    pub input: DocumentInput,
    pub command: Option<&'a str>,
    pub dependency_name: Option<&'a str>,
    pub selected_version: Option<&'a str>,
    pub responses: &'a [RegistryResponseInput],
}

impl VersionLensSession {
    pub async fn apply_command(
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

    pub async fn apply_command_with_selected_version(
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
            let parts = super::resolve_output_parts_with_plan(&input, edits, 0, vec![], 0);
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
                .map(|value| value.to_owned())
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
            edits,
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
                .find(|suggestion| selection::matches_dependency(&suggestion.dependency, selector))
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
            *suggestion = versionlens_suggestions::error(
                suggestion.dependency.clone(),
                "selected runtime version has not been verified".to_owned(),
            );
            continue;
        }
        let requirement =
            versionlens_model::registry_alias_requirement(&suggestion.dependency.requirement)
                .unwrap_or(&suggestion.dependency.requirement);
        let dialect = if suggestion.dependency.ecosystem == versionlens_model::Ecosystem::Python {
            versionlens_versions::VersionDialect::Pep440
        } else {
            versionlens_versions::VersionDialect::Semver
        };
        if versionlens_versions::normalized_version_for_dialect(version, dialect).is_some()
            && versionlens_versions::requirement_is_parseable_for_dialect(
                requirement,
                version,
                dialect,
            )
            && !versionlens_versions::is_update_available_for_dialect(version, requirement, dialect)
            && !versionlens_versions::requirement_satisfies_latest_for_dialect(
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
            .map(|value| value.to_owned()),
        hosted_name: suggestion
            .dependency
            .hosted_name
            .as_deref()
            .map(|value| value.to_owned()),
        range: suggestion.dependency.range,
        requirement_range: suggestion.dependency.requirement_range,
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

#[cfg(test)]
mod tests;
use std::time::Instant;
