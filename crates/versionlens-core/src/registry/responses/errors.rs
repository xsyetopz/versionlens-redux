use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::{RegistryErrorStatus, npm_error_status_from_response};
use versionlens_suggestions::{
    Suggestion, UpdateChoice, error, invalid, no_match_with_message, not_supported,
};

use crate::VersionLensSession;
use crate::model::RegistryResponseInput;
use crate::registry::registry_response_matches;

impl VersionLensSession {
    pub(crate) fn error_suggestion_from_responses(
        dependency: Dependency,
        responses: &[RegistryResponseInput],
    ) -> Result<Dependency, Box<Suggestion>> {
        if dependency.ecosystem != Ecosystem::Npm {
            return Ok(dependency);
        }

        match npm_error_status_for_dependency(&dependency, responses) {
            Some(RegistryErrorStatus::Error(message)) => Err(Box::new(error(dependency, message))),
            Some(RegistryErrorStatus::Invalid(message)) => {
                Err(Box::new(invalid(dependency, message)))
            }
            Some(RegistryErrorStatus::InvalidWithLatest(message)) => {
                let mut suggestion = invalid(dependency, message);
                suggestion.choices.push(UpdateChoice {
                    label: "latest".to_owned(),
                    version: "latest".to_owned(),
                    command: "update".to_owned(),
                });
                Err(Box::new(suggestion))
            }
            Some(RegistryErrorStatus::NoMatch(message)) => {
                Err(Box::new(no_match_with_message(dependency, Some(message))))
            }
            Some(RegistryErrorStatus::NotSupported) => Err(Box::new(not_supported(dependency))),
            None => Ok(dependency),
        }
    }
}

fn npm_error_status_for_dependency(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> Option<RegistryErrorStatus> {
    responses
        .iter()
        .filter(|response| registry_response_matches(response, dependency))
        .find_map(|response| npm_error_status_from_response(&response.body))
}
