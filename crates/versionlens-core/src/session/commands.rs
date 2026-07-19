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
use crate::command::{filter_update_command, project_version_bump};
use crate::contract::{RegistryResponseInput, ResolveDocumentOutput};
use crate::project::is_project_version_dependency;
use crate::status::to_u32;
use crate::suggestion::into_suggestion_payloads;

pub struct ApplyCommandRequest<'a> {
    pub input: DocumentInput,
    pub command: Option<&'a str>,
    pub dependency_name: Option<&'a str>,
    pub selected_version: Option<&'a str>,
    pub responses: &'a [RegistryResponseInput],
}

impl VersionLensSession {
    pub fn apply_command(
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
    }

    pub fn apply_command_with_selected_version(
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
        let operation = OperationContext::with_timeout(crate::duration_from_millis(
            self.config.http.timeout_ms,
        ));
        if command == Some("sort") {
            let dependencies = self.dependencies(&input);
            let edits = sort_dependency_edits(&input.text, &dependencies);
            return ResolveDocumentOutput {
                edits,
                ..empty_resolve_output()
            };
        }
        let selected_version = selected_version.filter(|_| recognized_update_command(command));

        let manifest_kind = self.classify_document(&input);
        let project_bump = project_version_bump(command, dependency_name);
        let mut suggestions = match dependency_name {
            Some(name) => self.resolve_dependency_suggestions(DependencySuggestionsRequest {
                input,
                selector: name,
                responses,
                project_bump,
                operation: &operation,
            }),
            None => self.resolve_suggestions(input, responses, project_bump, &operation),
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
            } else {
                (0, None, None)
            };
        let authorization_required_requests = operation.take_authorization_requests();
        let authorization_required_count =
            authorization_required_count.max(to_u32(authorization_required_requests.len()));
        let suggestion_payloads = into_suggestion_payloads(suggestions);
        ResolveDocumentOutput {
            suggestions: suggestion_payloads,
            edits,
            authorization_required_count,
            authorization_required_requests,
            vulnerable_update_count,
            vulnerable_update_package,
            vulnerable_update_version,
        }
    }

    pub(crate) fn vulnerable_update_count(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
    ) -> u32 {
        self.vulnerable_update_summary(suggestions, responses, manifest_kind, operation)
            .0
    }

    fn vulnerable_update_summary(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
    ) -> (u32, Option<String>, Option<String>) {
        for suggestion in suggestions {
            self.cache_update_choice_vulnerabilities(
                suggestion,
                responses,
                manifest_kind,
                operation,
            );
        }

        let mut count = 0;
        let mut package = None;
        let mut version = None;
        for suggestion in suggestions {
            let Some(dependency) = target_update_dependency(suggestion) else {
                continue;
            };
            self.cache_vulnerabilities(&dependency, responses, manifest_kind, operation);
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

    fn cache_update_choice_vulnerabilities(
        &self,
        suggestion: &Suggestion,
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
        operation: &OperationContext,
    ) {
        for choice in &suggestion.choices {
            let dependency = update_dependency_for_version(suggestion, choice.version.as_str());
            self.cache_vulnerabilities(&dependency, responses, manifest_kind, operation);
        }
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
    }
}

fn force_selected_version(suggestions: &mut [Suggestion], version: &str) {
    for suggestion in suggestions {
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
    }
}

#[cfg(test)]
mod tests;
