use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::docker_tag_exists;

use crate::RegistryResponseInput;
use crate::registry::registry_response_matches;

pub(crate) fn docker_response_missing_tag(
    dependency: &Dependency,
    responses: &[RegistryResponseInput],
) -> bool {
    dependency.ecosystem == Ecosystem::Docker
        && !dependency.requirement.is_empty()
        && responses
            .iter()
            .filter(|response| registry_response_matches(response, dependency))
            .find_map(|response| docker_tag_exists(&response.body, &dependency.requirement))
            == Some(false)
}

#[cfg(test)]
mod tests;
