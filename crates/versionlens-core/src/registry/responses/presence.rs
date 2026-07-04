use versionlens_parsers::Dependency;

use crate::VersionLensSession;
use crate::model::RegistryResponseInput;
use crate::registry::registry_response_matches;

impl VersionLensSession {
    pub(crate) fn has_registry_response(
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
    ) -> bool {
        responses
            .iter()
            .any(|response| registry_response_matches(response, dependency))
    }
}
