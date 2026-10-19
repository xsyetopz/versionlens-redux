use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{Hex, Npm};
use versionlens_providers::RegistryErrorStatus::{
    Error as RegistryStatusError, Invalid as RegistryStatusInvalid,
    InvalidWithLatest as RegistryStatusInvalidWithLatest, NoMatch as RegistryStatusNoMatch,
    NotSupported as RegistryStatusNotSupported,
};
use versionlens_providers::{
    RegistryErrorStatus, http_status_message_from_code, npm_error_status_from_response,
};
use versionlens_update::{
    Suggestion, UpdateChoice, error, invalid, no_match_with_message, not_supported, rate_limited,
};

use crate::RegistryResponseInput;
use crate::VersionLensSession;
use crate::registry::registry_response_matches;

impl VersionLensSession {
    pub(crate) fn error_suggestion_from_responses(
        dependency: Dependency,
        responses: &[RegistryResponseInput],
    ) -> Result<Dependency, Box<Suggestion>> {
        if !matches!(dependency.ecosystem, Npm | Hex) {
            return Ok(dependency);
        }

        match registry_error_status_for_dependency(&dependency, responses) {
            // The registry body says 429 but not when it resets.
            Some(RegistryStatusError(message))
                if Some(message.as_str()) == http_status_message_from_code(429) =>
            {
                Err(Box::new(rate_limited(
                    dependency,
                    "rate limited".to_owned(),
                )))
            }
            Some(RegistryStatusError(message)) => Err(Box::new(error(dependency, message))),
            Some(RegistryStatusInvalid(message)) => Err(Box::new(invalid(dependency, message))),
            Some(RegistryStatusInvalidWithLatest(message)) => {
                let mut suggestion = invalid(dependency, message);
                suggestion.choices.push(UpdateChoice {
                    label: "latest".to_owned(),
                    version: "latest".to_owned(),
                    command: "update".to_owned(),
                    replacement: None,
                });
                Err(Box::new(suggestion))
            }
            Some(RegistryStatusNoMatch(message)) => {
                Err(Box::new(no_match_with_message(dependency, Some(message))))
            }
            Some(RegistryStatusNotSupported) => Err(Box::new(not_supported(dependency))),
            None => Ok(dependency),
        }
    }
}

fn registry_error_status_for_dependency(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> Option<RegistryErrorStatus> {
    responses
        .iter()
        .filter(|response| registry_response_matches(response, dependency))
        .find_map(|response| npm_error_status_from_response(&response.body))
}
