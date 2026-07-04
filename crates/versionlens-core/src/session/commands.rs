use versionlens_parsers::{Dependency, DocumentInput, ManifestKind};
use versionlens_suggestions::{Suggestion, SuggestionStatus};

use crate::VersionLensSession;
use crate::command::{filter_update_command, project_version_bump};
use crate::model::{RegistryResponseInput, ResolveDocumentOutput};
use crate::project::is_project_version_dependency;
use crate::status::to_u32;
use crate::suggestion::into_suggestion_payloads;

impl VersionLensSession {
    pub fn apply_command(
        &self,
        input: DocumentInput,
        command: Option<&str>,
        dependency_name: Option<&str>,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        self.apply_command_with_selected_version(input, command, dependency_name, None, responses)
    }

    pub fn apply_command_with_selected_version(
        &self,
        input: DocumentInput,
        command: Option<&str>,
        dependency_name: Option<&str>,
        selected_version: Option<&str>,
        responses: &[RegistryResponseInput],
    ) -> ResolveDocumentOutput {
        self.clear_authorization_requests();
        if command == Some("sort") {
            let dependencies = self.dependencies(&input);
            let edits = versionlens_edits::sort_dependency_edits(&input.text, &dependencies);
            return ResolveDocumentOutput {
                suggestions: Vec::new(),
                edits,
                authorization_required_count: 0,
                authorization_required_requests: Vec::new(),
                vulnerable_update_count: 0,
                vulnerable_update_package: None,
                vulnerable_update_version: None,
            };
        }

        let manifest_kind = self.classify_document(&input);
        let project_bump = project_version_bump(command, dependency_name);
        let mut suggestions = match dependency_name {
            Some(name) => self.resolve_dependency_suggestions(input, name, responses, project_bump),
            None => self.resolve_suggestions(input, responses, project_bump),
        };
        let bulk_dependency_update = bulk_dependency_update_command(command, dependency_name);
        if bulk_dependency_update {
            suggestions.retain(|suggestion| !is_project_version_dependency(&suggestion.dependency));
        }
        if let Some(version) = selected_version {
            force_selected_version(&mut suggestions, version);
        }
        filter_update_command(&mut suggestions, command, selected_version.is_some());
        let edits = if bulk_dependency_update {
            versionlens_edits::bulk_update_edits(&suggestions)
        } else {
            versionlens_edits::update_edits(&suggestions)
        };
        let authorization_required_count = Self::authorization_required_count(&suggestions);
        let (vulnerable_update_count, vulnerable_update_package, vulnerable_update_version) =
            if self.config.show_vulnerabilities && dependency_name.is_some() {
                self.vulnerable_update_summary(&suggestions, responses, Some(manifest_kind))
            } else {
                (0, None, None)
            };
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
            vulnerable_update_package,
            vulnerable_update_version,
        }
    }

    pub(crate) fn vulnerable_update_count(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
    ) -> u32 {
        self.vulnerable_update_summary(suggestions, responses, manifest_kind)
            .0
    }

    fn vulnerable_update_summary(
        &self,
        suggestions: &[Suggestion],
        responses: &[RegistryResponseInput],
        manifest_kind: Option<ManifestKind>,
    ) -> (u32, Option<String>, Option<String>) {
        for suggestion in suggestions {
            self.cache_update_choice_vulnerabilities(suggestion, responses, manifest_kind);
        }

        let mut count = 0;
        let mut package = None;
        let mut version = None;
        for suggestion in suggestions {
            let Some(dependency) = target_update_dependency(suggestion) else {
                continue;
            };
            self.cache_vulnerabilities(&dependency, responses, manifest_kind);
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
    ) {
        for choice in &suggestion.choices {
            let dependency = update_dependency_for_version(suggestion, choice.version.as_str());
            self.cache_vulnerabilities(&dependency, responses, manifest_kind);
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
                suggestion.status == SuggestionStatus::Error
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

fn force_selected_version(suggestions: &mut [Suggestion], version: &str) {
    for suggestion in suggestions {
        suggestion.latest = Some(version.to_owned());
        suggestion.status = SuggestionStatus::UpdateAvailable;
    }
}

fn target_update_dependency(suggestion: &Suggestion) -> Option<Dependency> {
    let latest = suggestion.latest.as_deref()?;
    (suggestion.status == SuggestionStatus::UpdateAvailable
        || suggestion.status == SuggestionStatus::BuildAvailable)
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
            .map(str::to_owned),
        hosted_name: suggestion
            .dependency
            .hosted_name
            .as_deref()
            .map(str::to_owned),
        range: suggestion.dependency.range,
        requirement_range: suggestion.dependency.requirement_range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}

#[cfg(test)]
mod tests;
