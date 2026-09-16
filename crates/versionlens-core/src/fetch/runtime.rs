use super::latest::LatestFetch;
use crate::{
    RegistryResponseInput, VersionLensSession, error::FetchError,
    registry::registry_response_matches,
};
use versionlens_model::{Dependency, Ecosystem};
use versionlens_providers::RuntimeSource;
use versionlens_suggestions::UpdateChoice;
use versionlens_versions::{
    VersionDialect, latest_version, latest_version_for_dialect, normalized_version_for_dialect,
    requirement_satisfies_latest, requirement_satisfies_latest_for_dialect,
};

mod body;
mod integrity;

#[cfg(test)]
mod tests;

impl VersionLensSession {
    pub(crate) async fn fetch_runtime_latest(
        &self,
        dependency: &Dependency,
        responses: &[RegistryResponseInput],
        context: &crate::registry::RegistryContext,
        operation: &crate::session::operation::OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        if dependency.hosted_url.as_deref() == Some("runtime-expression") {
            return Err(runtime_error(
                "runtime input requires a literal value or a finite matrix",
            ));
        }
        let source = RuntimeSource::for_dependency(dependency).map_err(runtime_error)?;
        let body = if let Some(response) = responses
            .iter()
            .find(|response| registry_response_matches(response, dependency))
        {
            response.body.clone()
        } else {
            self.fetch_runtime_body(dependency, &source, context, operation)
                .await?
        };
        let versions = source
            .versions(&body, &dependency.requirement)
            .map_err(runtime_error)?;
        let channel = source.is_channel(&dependency.requirement);
        let constraint =
            dependency.is_runtime_constraint() || source.is_constraint(&dependency.requirement);
        let preferred = if constraint && !channel {
            None
        } else {
            source
                .preferred_release(&body, &dependency.requirement)
                .map_err(runtime_error)?
        };
        let selected = if channel {
            if preferred.is_some() {
                preferred
            } else if matches!(source, RuntimeSource::BunCanary) {
                versions.first().cloned()
            } else {
                latest_version(versions.iter().map(String::as_str), true)
            }
        } else {
            let compatible = versions
                .iter()
                .filter(|version| runtime_matches(dependency, version))
                .collect::<Vec<_>>();
            if compatible.is_empty() {
                return Err(runtime_error(format!(
                    "no published {} release satisfies {}",
                    dependency.name, dependency.requirement
                )));
            }
            let candidates = if constraint {
                compatible
            } else if let Some(preferred) = preferred.as_ref() {
                versions
                    .iter()
                    .filter(|version| *version == preferred || runtime_matches(dependency, version))
                    .collect()
            } else {
                versions.iter().collect()
            };
            latest_version_for_dialect(
                candidates.into_iter().map(String::as_str),
                self.includes_prereleases(dependency),
                &[],
                runtime_dialect(dependency),
            )
        };
        let latest = selected
            .ok_or_else(|| runtime_error("runtime release source contains no matching version"))?;
        let replacement = self
            .runtime_replacement(integrity::RuntimeReplacementRequest {
                dependency,
                source: &source,
                body: &body,
                selected: &latest,
                context,
                operation,
            })
            .await?;
        let choices = if constraint || channel || runtime_exactly_equals(dependency, &latest) {
            vec![]
        } else {
            vec![UpdateChoice {
                label: "latest".to_owned(),
                version: latest.clone(),
                command: "update".to_owned(),
                replacement,
            }]
        };
        Ok(LatestFetch {
            latest: Some(latest),
            builds: vec![],
            choices,
        })
    }
}

fn runtime_exactly_equals(dependency: &Dependency, version: &str) -> bool {
    let requirement = dependency
        .requirement
        .split_once("+sha")
        .map_or(dependency.requirement.as_str(), |(version, _)| version)
        .trim_start_matches(['v', 'V']);
    let dialect = runtime_dialect(dependency);
    let exact = if dialect == VersionDialect::Pep440 {
        requirement.split('.').count() >= 3
            && requirement
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    } else {
        semver::Version::parse(requirement).is_ok()
    };
    exact
        && normalized_version_for_dialect(requirement, dialect)
            == normalized_version_for_dialect(version, dialect)
}

fn runtime_matches(dependency: &Dependency, version: &str) -> bool {
    let requirement = dependency
        .requirement
        .split_once("+sha")
        .map_or(dependency.requirement.as_str(), |(version, _)| version);
    if dependency.group == "rust-version" {
        return matches!(
            versionlens_versions::compare_versions(version, requirement),
            Some(std::cmp::Ordering::Equal | std::cmp::Ordering::Greater)
        );
    }
    if versionlens_versions::compare_versions(version, requirement)
        == Some(std::cmp::Ordering::Equal)
    {
        return true;
    }
    let parts = requirement
        .trim_start_matches('v')
        .split('.')
        .collect::<Vec<_>>();
    if parts.len() < 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        let published = version.split('.').collect::<Vec<_>>();
        return parts
            .iter()
            .enumerate()
            .all(|(index, part)| published.get(index) == Some(part));
    }
    if runtime_dialect(dependency) == VersionDialect::Pep440 {
        requirement_satisfies_latest_for_dialect(requirement, version, VersionDialect::Pep440)
    } else {
        requirement_satisfies_latest(requirement, version)
    }
}

fn runtime_dialect(dependency: &Dependency) -> VersionDialect {
    if dependency.name == "java"
        || dependency.ecosystem == Ecosystem::Python && !dependency.group.starts_with("with.")
    {
        VersionDialect::Pep440
    } else {
        VersionDialect::Semver
    }
}

fn runtime_error(message: impl Into<String>) -> FetchError {
    FetchError::RegistryStatus(message.into())
}
