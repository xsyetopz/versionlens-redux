use semver::Version;
use versionlens_model::Dependency;
use versionlens_providers::{
    LatestVersionRequest, RegistryEndpoint, build_versions_from_response,
    latest_version_from_response_for_endpoint, release_versions_from_response_for_endpoint,
    release_versions_from_response_for_package,
};
use versionlens_update::{UpdateChoice, release_update_choices_with_prereleases, version_dialect};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;
use versionlens_model::Ecosystem::{Docker, Npm};
mod docker;
mod github;
use docker::docker_update_choices;
use github::{attach_github_action_replacements, github_action_versions};
pub(crate) use github::{github_action_latest, github_current_ref_is_proven};

mod body;
mod local_dotnet;

pub(crate) struct LatestFetch {
    pub(crate) latest: Option<String>,
    pub(crate) builds: Vec<String>,
    pub(crate) choices: Vec<UpdateChoice>,
}

type UpdateChoices = Vec<UpdateChoice>;

#[derive(Clone, Copy)]
struct ResponseUpdateRequest<'a> {
    dependency: &'a Dependency,
    endpoint: Option<&'a RegistryEndpoint>,
    latest: &'a str,
    body: &'a str,
    include_prereleases: bool,
    prerelease_tags: &'a [String],
}

impl VersionLensSession {
    pub(crate) async fn fetch_latest(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        let mut first_error = None;

        for endpoint in self.registry_endpoints_with_context(dependency, context) {
            if operation.is_expired() {
                return Err(FetchError::OperationTimeout);
            }
            match self
                .fetch_latest_from_endpoint(dependency, &endpoint, context, operation)
                .await
            {
                Ok(fetch) if fetch.latest.is_some() => return Ok(fetch),
                Ok(_) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                }
            }
        }

        match first_error {
            Some(error) => Err(error),
            None => Ok(LatestFetch {
                latest: None,
                builds: vec![],
                choices: vec![],
            }),
        }
    }

    async fn fetch_latest_from_endpoint(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> Result<LatestFetch, FetchError> {
        let Some(body) = self
            .fetch_registry_body(dependency, &endpoint.url, context, operation)
            .await?
        else {
            return Ok(LatestFetch {
                latest: None,
                builds: vec![],
                choices: vec![],
            });
        };

        let current_ref_is_proven = if github_current_ref_is_proven(dependency, &body) {
            true
        } else if github::is_commit_pin(dependency) {
            self.fetch_commit_pin_is_proven(dependency, endpoint, context, operation)
                .await?
        } else {
            self.fetch_exact_github_action_ref_is_proven(dependency, endpoint, context, operation)
                .await?
        };
        let latest = current_ref_is_proven
            .then(|| {
                github_action_latest(
                    dependency,
                    &body,
                    self.includes_prereleases(dependency),
                    self.prerelease_tags(dependency.ecosystem),
                )
                .or_else(|| self.latest_from_fetched_body(dependency, endpoint, &body))
            })
            .flatten();
        let choices = latest
            .as_deref()
            .map(|version| {
                response_update_choices_with_endpoint(ResponseUpdateRequest {
                    dependency,
                    endpoint: Some(endpoint),
                    latest: version,
                    body: &body,
                    include_prereleases: self.includes_prereleases(dependency),
                    prerelease_tags: self.prerelease_tags(dependency.ecosystem),
                })
            })
            .unwrap_or_default();
        Ok(LatestFetch {
            latest,
            builds: build_versions_from_response(
                dependency.ecosystem,
                &body,
                &dependency.requirement,
            ),
            choices,
        })
    }
}

pub(crate) fn response_update_choices(
    dependency: &Dependency,
    latest: &str,
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> UpdateChoices {
    response_update_choices_with_endpoint(ResponseUpdateRequest {
        dependency,
        endpoint: None,
        latest,
        body,
        include_prereleases,
        prerelease_tags,
    })
}

fn response_update_choices_with_endpoint(request: ResponseUpdateRequest<'_>) -> UpdateChoices {
    let ResponseUpdateRequest {
        dependency,
        endpoint,
        latest,
        body,
        include_prereleases,
        prerelease_tags,
    } = request;
    if dependency.ecosystem == Docker {
        return docker_update_choices(&dependency.requirement, latest, body);
    }

    let versions = github_action_versions(dependency, body).unwrap_or_else(|| {
        update_choice_versions_from_response(dependency, endpoint, body, latest)
    });
    let current_tag = dependency
        .canonical_reference
        .as_ref()
        .filter(|reference| {
            matches!(
                reference,
                versionlens_model::CanonicalReference::GitHubActionCommit { .. }
            )
        })
        .and_then(|reference| github::github_action_tag(reference, body));
    let requirement = current_tag
        .as_deref()
        .and_then(versionlens_model::version_tag_parts)
        .map_or(dependency.requirement.as_str(), |(_, version)| version);
    let mut choices = release_update_choices_with_prereleases(
        requirement,
        latest,
        &versions,
        include_prereleases,
        prerelease_tags,
        version_dialect(dependency),
    );
    if current_tag.is_some() && !choices.iter().any(|choice| choice.version == latest) {
        choices.push(UpdateChoice {
            label: "latest".to_owned(),
            version: latest.to_owned(),
            command: "update".to_owned(),
            replacement: None,
        });
    }
    attach_github_action_replacements(dependency, body, &mut choices);
    choices
}

fn update_choice_versions_from_response(
    dependency: &Dependency,
    endpoint: Option<&RegistryEndpoint>,
    body: &str,
    latest: &str,
) -> Vec<String> {
    let versions = registry_release_versions(dependency, endpoint, body);
    if dependency.ecosystem == Npm {
        return npm_versions_capped_to_latest(versions, latest);
    }
    versions
}

fn registry_release_versions(
    dependency: &Dependency,
    endpoint: Option<&RegistryEndpoint>,
    body: &str,
) -> Vec<String> {
    let package = dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name);
    endpoint.map_or_else(
        || release_versions_from_response_for_package(dependency.ecosystem, package, body),
        |endpoint| {
            release_versions_from_response_for_endpoint(
                endpoint,
                dependency.ecosystem,
                package,
                body,
            )
        },
    )
}

fn npm_versions_capped_to_latest(versions: Vec<String>, latest: &str) -> Vec<String> {
    let Some(latest) = stable_semver(latest) else {
        return versions;
    };

    versions
        .into_iter()
        .filter(|version| {
            let Some(parsed) = crate::parse_semver(version).ok() else {
                return true;
            };
            !parsed.pre.is_empty() || semver_precedence_lte(&parsed, &latest)
        })
        .collect()
}

fn stable_semver(version: &str) -> Option<Version> {
    let version = crate::parse_semver(version.trim()).ok()?;
    version.pre.is_empty().then_some(version)
}

fn semver_precedence_lte(version: &Version, latest: &Version) -> bool {
    (version.major, version.minor, version.patch) <= (latest.major, latest.minor, latest.patch)
}

impl VersionLensSession {
    pub(in crate::fetch::latest) fn latest_from_fetched_body(
        &self,
        dependency: &Dependency,
        endpoint: &RegistryEndpoint,
        body: &str,
    ) -> Option<String> {
        latest_version_from_response_for_endpoint(
            endpoint,
            LatestVersionRequest {
                ecosystem: dependency.ecosystem,
                package: dependency
                    .hosted_name
                    .as_deref()
                    .unwrap_or(&dependency.name),
                requirement: &dependency.requirement,
                body,
                include_prereleases: self.includes_prereleases(dependency),
                prerelease_tags: self.prerelease_tags(dependency.ecosystem),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use versionlens_model::CanonicalReference;
    use versionlens_model::DocumentInput;
    use versionlens_model::{Dependency, Ecosystem};
    use versionlens_model::{Position, Range};

    use super::github::github_action_reference_is_proven_by_exact_ref;
    use super::response_update_choices;
    use crate::{ProviderSettings, RegistryUrlConfig};
    use versionlens_model::Ecosystem::*;

    #[tokio::test]
    async fn invalid_registry_url_creates_contextual_error_suggestion() {
        let session = crate::test_support::session_with_provider_settings(
            ProviderSettings {
                registry_urls: vec![RegistryUrlConfig {
                    ecosystem: Npm,
                    url: "not a url".to_owned(),
                }],
                ..Default::default()
            },
            false,
        );

        let output = session
            .resolve_document(DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "invalid-registry-url-creates-contextual-error-suggestion.json",
                ),
                None,
            ))
            .await;

        assert_eq!(output.suggestions[0].status, "error");
        assert!(
            output.suggestions[0]
                .latest
                .as_deref()
                .is_some_and(|message| message.contains("failed to fetch registry URL")),
        );
    }

    #[test]
    fn cran_update_choices_exclude_versions_from_other_packages() {
        let dependency = update_choice_dependency("dplyr", Cran, "Imports");
        let body = "Package: dplyr\nVersion: 1.0.0\n\nPackage: dplyr\nVersion: 1.1.4\n\nPackage: unrelated\nVersion: 2.0.0\n";

        let choices = response_update_choices(&dependency, "1.1.4", body, false, &[]);

        assert_eq!(
            choices
                .iter()
                .map(|choice| choice.version.as_str())
                .collect::<Vec<_>>(),
            ["1.1.4"]
        );
    }

    #[test]
    fn composer_update_choices_exclude_versions_from_other_packages() {
        let dependency = update_choice_dependency("acme/target", Composer, "require");
        let body = r#"{
      "packages": {
        "acme/target": [{"version": "1.0.0"}, {"version": "1.1.0"}],
        "acme/unrelated": [{"version": "9.0.0"}]
      }
    }"#;

        let choices = response_update_choices(&dependency, "1.1.0", body, false, &[]);

        assert_eq!(
            choices
                .iter()
                .map(|choice| choice.version.as_str())
                .collect::<Vec<_>>(),
            ["1.1.0"]
        );
    }

    #[test]
    fn annotated_github_tag_objects_require_commit_dereferencing() {
        let reference = CanonicalReference::GitHubActionSha {
            commit: "8216e11d8cd9b42fe925c852af8e76311ff067ac".to_owned(),
            tag: "v2".to_owned(),
            separator: " # ".to_owned(),
        };
        let body = r#"{
      "ref": "refs/tags/v2",
      "object": {
        "type": "tag",
        "sha": "8216e11d8cd9b42fe925c852af8e76311ff067ac"
      }
    }"#;

        assert!(!github_action_reference_is_proven_by_exact_ref(
            &reference, body
        ));
    }

    #[test]
    fn exact_github_ref_must_match_both_annotation_and_sha() {
        let reference = CanonicalReference::GitHubActionSha {
            commit: "8216e11".to_owned(),
            tag: "v2".to_owned(),
            separator: " # ".to_owned(),
        };

        assert!(!github_action_reference_is_proven_by_exact_ref(
            &reference,
            r#"{"ref":"refs/tags/v3","object":{"type":"tag","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
        ));
        assert!(!github_action_reference_is_proven_by_exact_ref(
            &reference,
            r#"{"ref":"refs/tags/v2","object":{"type":"tag","sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}"#
        ));
        assert!(github_action_reference_is_proven_by_exact_ref(
            &reference,
            r#"{"ref":"refs/tags/v2","object":{"type":"commit","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
        ));
        assert!(!github_action_reference_is_proven_by_exact_ref(
            &reference,
            r#"{"ref":"refs/tags/v2","object":{"sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#
        ));
        for sha in ["8216e11", "8216e11not-a-commit-object-sha0000000000000"] {
            let body =
                serde_json::json!({"ref":"refs/tags/v2","object":{"type":"commit","sha":sha}})
                    .to_string();
            assert!(!github_action_reference_is_proven_by_exact_ref(
                &reference, &body
            ));
        }
    }

    #[test]
    fn action_commit_families_require_complete_commits_and_ignore_hex_case() {
        let reference = CanonicalReference::GitHubActionCommit {
            commit: "abcdef0".to_owned(),
        };
        for (sha, expected) in [
            ("abcdef0", None),
            ("abcdef000000000000000000000000000000000x", None),
            ("ABCDEF0000000000000000000000000000000000", Some("v1.1.0")),
        ] {
            let body = serde_json::json!([
                {"name":"v1.0.0","commit":{"sha":sha.to_ascii_lowercase()}},
                {"name":"v1.1.0","commit":{"sha":sha}}
            ])
            .to_string();
            assert_eq!(
                super::github::github_action_tag(&reference, &body).as_deref(),
                expected
            );
        }
    }

    fn update_choice_dependency(name: &str, ecosystem: Ecosystem, group: &str) -> Dependency {
        Dependency {
            name: name.to_owned(),
            requirement: "1.0.0".to_owned(),
            ecosystem,
            group: group.to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: empty_range(),
            requirement_range: empty_range(),
            requirement_prefix: String::new(),
            requirement_suffix: String::new(),
            canonical_reference: None,
        }
    }

    fn empty_range() -> Range {
        let position = Position {
            line: 0,
            character: 0,
        };
        Range {
            start: position,
            end: position,
        }
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/core-scenarios/fetch/latest/tests", name)
    }

    fn serve_once(status: u16, body: &'static str) -> String {
        use std::io::{Read, Write};
        let listener = crate::test_support::tcp_listener_bind("127.0.0.1:0").unwrap();
        let url = format!(
            "http://{}/repos/owner/repo/commits",
            listener.local_addr().unwrap()
        );
        std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request).unwrap();
            let response = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        });
        url
    }

    #[tokio::test]
    async fn commit_pin_lookup_proves_only_a_matching_sha() {
        let session = crate::test_support::test_session(false);
        let mut dependency = update_choice_dependency("owner/repo", Ruby, "default");
        dependency.requirement = "deadbee".to_owned();
        for (status, body, proven) in [
            (
                200,
                r#"{"sha":"deadbee000000000000000000000000000000000"}"#,
                true,
            ),
            (
                200,
                r#"{"sha":"0123456789abcdef0123456789abcdef01234567"}"#,
                false,
            ),
            (
                404,
                r#"{"message":"No commit found for SHA: deadbee"}"#,
                false,
            ),
        ] {
            let endpoint =
                versionlens_providers::RegistryEndpoint::ecosystem(serve_once(status, body));
            let result = session
                .fetch_commit_pin_is_proven(
                    &dependency,
                    &endpoint,
                    &crate::registry::RegistryContext::default(),
                    &crate::session::operation::OperationContext::default(),
                )
                .await
                .unwrap();
            assert_eq!(result, proven, "{body}");
        }
    }
}
