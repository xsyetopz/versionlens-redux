use versionlens_model::Dependency;

use crate::RegistryResponseInput;

pub(crate) fn registry_response_matches(
    response: &RegistryResponseInput,
    dependency: &Dependency,
) -> bool {
    response.ecosystem == dependency.ecosystem
        && (response.package == dependency.name
            || dependency
                .hosted_name
                .as_deref()
                .is_some_and(|hosted_name| response.package == hosted_name))
}
