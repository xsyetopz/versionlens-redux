use crate::prerelease;
use semver::Version;
use serde_json::Value;
use serde_json::from_str;
use versionlens_model::Ecosystem::{Composer, Docker, Dotnet};
use versionlens_model::{Dependency, DocumentInput};
use versionlens_providers::{
    is_registry_dependency, is_unsupported_dotnet_requirement,
    release_versions_from_response_for_package,
};
use versionlens_suggestions::SuggestionStatus::{
    BuildAvailable as StatusBuildAvailable, Current as StatusCurrent, Satisfies as StatusSatisfies,
    UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_suggestions::{
    Suggestion, UpdateChoice, error, fixed, invalid, no_match, resolve_dependency,
};
use versionlens_versions::{ProjectVersionBump, is_build_update, is_dotnet_requirement_parseable};

use crate::RegistryResponseInput;
use crate::VersionLensSession;
use crate::docker::response::docker_response_missing_tag;
use crate::non_registry::{deno_import_has_no_suggestions, known_non_registry_suggestion};
use crate::project::project_version_latest;
use crate::registry::{RegistryContext, registry_response_matches};
use crate::session::operation::OperationContext;
use crate::workspace::WorkspaceGraph;

use super::latest::{LatestLookup, LatestResolutionRequest};

type UpdateChoices = Vec<UpdateChoice>;

pub(super) struct ResolveDependencyInput<'a> {
    pub(super) dependency: Dependency,
    pub(super) document: &'a DocumentInput,
    pub(super) document_uri: Option<&'a str>,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) project_bump: Option<ProjectVersionBump>,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}

struct LatestSuggestionRequest<'a> {
    dependency: Dependency,
    responses: &'a [RegistryResponseInput],
    has_registry_response: bool,
    context: &'a RegistryContext,
    operation: &'a OperationContext,
}

impl VersionLensSession {
    pub(super) fn resolve_dependency_with_responses(
        &self,
        input: ResolveDependencyInput<'_>,
    ) -> Option<Suggestion> {
        let ResolveDependencyInput {
            dependency,
            document,
            document_uri,
            responses,
            project_bump,
            context,
            operation,
        } = input;

        if let Some(latest) = project_version_latest(&dependency, project_bump) {
            return Some(resolve_dependency(dependency, Some(latest)));
        }

        // Proven local identities are authoritative; ambiguous/malformed
        // workspace members deliberately fall through to existing behavior.
        if let Some(local) = WorkspaceGraph::for_document(document).resolve(&dependency) {
            return Some(resolve_dependency(dependency, Some(local.version)));
        }

        if deno_import_has_no_suggestions(&dependency) {
            return None;
        }

        let dependency = match known_non_registry_suggestion(dependency, document_uri) {
            Ok(suggestion) => return Some(suggestion),
            Err(dependency) => *dependency,
        };
        if !is_registry_dependency(
            dependency.ecosystem,
            &dependency.name,
            &dependency.requirement,
        ) {
            return Some(resolve_dependency(dependency, None));
        }
        if invalid_composer_registry_requirement(&dependency) {
            return Some(invalid(dependency, "invalid version".to_owned()));
        }
        if let Some(version) = context.composer_inline_package_version(&dependency) {
            return Some(resolve_dependency(dependency, Some(version)));
        }

        self.registry_dependency_suggestion(dependency, responses, context, operation)
    }

    fn registry_dependency_suggestion(
        &self,
        dependency: Dependency,
        responses: &[RegistryResponseInput],
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Option<Suggestion> {
        if self.config.show_vulnerabilities {
            self.cache_vulnerabilities(&dependency, responses, context.manifest_kind(), operation);
        }

        if is_unsupported_dotnet_registry_dependency(&dependency) {
            return None;
        }

        let has_registry_response = Self::has_registry_response(&dependency, responses);
        let dependency = match Self::error_suggestion_from_responses(dependency, responses) {
            Ok(dependency) => dependency,
            Err(suggestion) => return Some(*suggestion),
        };
        if docker_response_missing_tag(&dependency, responses) {
            return Some(self.docker_no_match_suggestion(LatestSuggestionRequest {
                dependency,
                responses,
                has_registry_response,
                context,
                operation,
            }));
        }

        Some(self.latest_lookup_suggestion(LatestSuggestionRequest {
            dependency,
            responses,
            has_registry_response,
            context,
            operation,
        }))
    }

    fn docker_no_match_suggestion(&self, request: LatestSuggestionRequest<'_>) -> Suggestion {
        let lookup = self.resolve_latest_for_request(&request);
        let dependency = request.dependency;
        if let Some(message) = lookup.fetch_error {
            return error(dependency, message.to_string());
        }

        let mut suggestion = no_match(dependency);
        suggestion.builds = lookup.builds;
        suggestion.choices = lookup.choices;
        suggestion
    }

    fn latest_lookup_suggestion(&self, request: LatestSuggestionRequest<'_>) -> Suggestion {
        let lookup = self.resolve_latest_for_request(&request);
        let LatestSuggestionRequest {
            dependency,
            responses,
            has_registry_response,
            ..
        } = request;
        if let Some(message) = lookup.fetch_error {
            return error(dependency, message.to_string());
        }

        if docker_empty_requirement_resolves_to_non_numeric_latest(
            &dependency,
            lookup.latest.as_deref(),
        ) {
            let mut suggestion = no_match(dependency);
            suggestion.choices = docker_latest_tag_choices(lookup.choices);
            return suggestion;
        }

        if npm_dist_tag_missing_from_responses(&dependency, responses) {
            let mut suggestion = no_match(dependency);
            suggestion.choices = lookup.choices;
            return suggestion;
        }

        if fixed_requirement_missing_from_responses(&dependency, responses) {
            let mut suggestion = no_match(dependency);
            suggestion.builds = lookup.builds;
            suggestion.choices = lookup.choices;
            return suggestion;
        }

        if invalid_dotnet_registry_requirement(&dependency) {
            let mut suggestion = no_match(dependency);
            suggestion.choices = dotnet_invalid_requirement_choices(lookup.latest);
            return suggestion;
        }

        match lookup.latest {
            Some(latest)
                if fixed_requirement_matches_response(&dependency, responses)
                    && dependency.canonical_reference.is_none()
                    && !latest_matches_fixed_current(&dependency, latest.as_str())
                    && !is_build_update(latest.as_str(), &dependency.requirement) =>
            {
                let current = dependency.requirement.trim().to_owned();
                let mut suggestion = fixed(dependency, current);
                suggestion.builds = lookup.builds;
                suggestion.choices = lookup.choices;
                suggestion
            }
            Some(latest) => with_lookup_choices(
                resolve_dependency(dependency, Some(latest)),
                lookup.builds,
                lookup.choices,
            ),
            None if has_registry_response => no_match(dependency),
            None => resolve_dependency(dependency, None),
        }
    }

    fn resolve_latest_for_request(&self, request: &LatestSuggestionRequest<'_>) -> LatestLookup {
        self.resolve_latest(LatestResolutionRequest {
            dependency: &request.dependency,
            responses: request.responses,
            has_registry_response: request.has_registry_response,
            context: request.context,
            operation: request.operation,
        })
    }
}

fn with_lookup_choices(
    mut suggestion: Suggestion,
    builds: Vec<String>,
    choices: UpdateChoices,
) -> Suggestion {
    if suggestion.status == StatusBuildAvailable && !builds.is_empty() {
        suggestion.status = StatusCurrent;
        suggestion.latest = Some(suggestion.dependency.requirement.trim().to_owned());
    }
    if suggestion.status == StatusUpdateAvailable
        && docker_latest_alias_is_current(&suggestion, &builds)
    {
        if suggestion.dependency.requirement.trim() == "latest" {
            suggestion.latest = Some("latest".to_owned());
        }
        suggestion.status = StatusCurrent;
    }
    if suggestion.status == StatusUpdateAvailable && has_bump_choice(&choices) {
        suggestion.status = StatusSatisfies;
    }
    if docker_explicit_latest_alias_is_current(&suggestion, &builds) {
        suggestion.latest = Some("latest".to_owned());
    }
    suggestion.builds = builds;
    suggestion.choices = choices;
    suggestion
}

fn docker_latest_alias_is_current(suggestion: &Suggestion, builds: &[String]) -> bool {
    if suggestion.dependency.ecosystem != Docker {
        return false;
    }

    let current = suggestion.dependency.requirement.trim();
    if current.is_empty() {
        return suggestion
            .latest
            .as_deref()
            .is_some_and(docker_plain_numeric_tag);
    }

    builds.iter().any(|build| build.as_str() == current)
}

fn docker_explicit_latest_alias_is_current(suggestion: &Suggestion, builds: &[String]) -> bool {
    suggestion.status == StatusCurrent
        && suggestion.dependency.ecosystem == Docker
        && suggestion.dependency.requirement.trim() == "latest"
        && builds.iter().any(|build| build == "latest")
}

fn docker_empty_requirement_resolves_to_non_numeric_latest(
    dependency: &Dependency,
    latest: Option<&str>,
) -> bool {
    dependency.ecosystem == Docker
        && dependency.requirement.trim().is_empty()
        && latest.is_some_and(|latest| !docker_plain_numeric_tag(latest))
}

fn docker_plain_numeric_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn docker_latest_tag_choices(choices: UpdateChoices) -> UpdateChoices {
    choices
        .into_iter()
        .map(|mut choice| {
            if choice.label == "latest" {
                choice.version = "latest".to_owned();
            }
            choice
        })
        .collect()
}

fn has_bump_choice(choices: &[UpdateChoice]) -> bool {
    choices.iter().any(|choice| choice.label.as_str() == "bump")
}

fn is_unsupported_dotnet_registry_dependency(dependency: &Dependency) -> bool {
    dependency.ecosystem == Dotnet && is_unsupported_dotnet_requirement(&dependency.requirement)
}

fn invalid_dotnet_registry_requirement(dependency: &Dependency) -> bool {
    dependency.ecosystem == Dotnet && !is_dotnet_requirement_parseable(&dependency.requirement)
}

fn dotnet_invalid_requirement_choices(latest: Option<String>) -> UpdateChoices {
    latest
        .map(|version| {
            vec![UpdateChoice {
                label: "latest".to_owned(),
                version,
                command: "update".to_owned(),
                replacement: None,
            }]
        })
        .unwrap_or_default()
}

fn invalid_composer_registry_requirement(dependency: &Dependency) -> bool {
    dependency.ecosystem == Composer
        && !versionlens_versions::composer_requirement_is_parseable(&dependency.requirement)
}

fn npm_dist_tag_missing_from_responses(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> bool {
    let requirement = dependency.requirement.trim();
    prerelease::npm_requirement_may_be_dist_tag(dependency)
        && crate::parse_semver(requirement).is_err()
        && crate::parse_semver_req(requirement).is_err()
        && responses
            .iter()
            .filter(|response| registry_response_matches(response, dependency))
            .any(|response| npm_dist_tags_missing_requirement(&response.body, requirement))
}

fn npm_dist_tags_missing_requirement(body: &str, requirement: &str) -> bool {
    from_str::<Value>(body)
        .ok()
        .and_then(|value| {
            value
                .get("dist-tags")?
                .as_object()
                .map(|tags| !tags.contains_key(requirement))
        })
        .unwrap_or(false)
}

fn fixed_requirement_missing_from_responses(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> bool {
    if dependency.canonical_reference.is_some() {
        return false;
    }
    let Some((current, releases)) = fixed_response_releases(dependency, responses) else {
        return false;
    };
    !releases.is_empty()
        && !releases
            .iter()
            .any(|release| fixed_release_matches(release, &current))
}

fn fixed_requirement_matches_response(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> bool {
    let Some((current, releases)) = fixed_response_releases(dependency, responses) else {
        return false;
    };

    releases
        .iter()
        .any(|release| fixed_release_matches(release, &current))
}

fn fixed_response_releases(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> Option<(Version, Vec<String>)> {
    let current = fixed_current(dependency)?;
    let response = responses
        .iter()
        .find(|response| registry_response_matches(response, dependency))?;
    let releases = release_versions_from_response_for_package(
        dependency.ecosystem,
        dependency
            .hosted_name
            .as_deref()
            .unwrap_or(&dependency.name),
        &response.body,
    );
    Some((current, releases))
}

fn fixed_current(dependency: &Dependency) -> Option<Version> {
    crate::parse_semver(dependency.requirement.trim()).ok()
}

fn latest_matches_fixed_current(dependency: &Dependency, latest: &str) -> bool {
    fixed_current(dependency).is_some_and(|current| fixed_release_matches(latest, &current))
}

fn fixed_release_matches(release: &str, current: &Version) -> bool {
    crate::parse_semver(release.trim().trim_start_matches(['v', 'V']))
        .is_ok_and(|release| release.eq(current))
}

#[cfg(test)]
mod tests;
