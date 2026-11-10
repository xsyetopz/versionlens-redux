use versionlens_model::Ecosystem::{
    Composer, Cran, Docker, Dotnet, GitHub, Hex, Maven, Npm, Python, Ruby,
};
use versionlens_model::{Ecosystem, compare_versions};
pub(crate) mod cargo;
mod composer;
mod conan;
mod cpp;
mod cran;
mod deno;
mod dispatch;
mod docker;
mod dotnet;
mod dub;
mod endpoint;
mod errors;
mod github;
mod go;
mod hackage;
mod haxelib;
mod helm;
mod hex;
mod julia;
mod luarocks;
mod nim;
mod npm;
mod opam;
mod pub_registry;
mod python;
mod python_simple;
mod ruby;
mod swift;
mod vcpkg;
mod versions;
mod xml;

pub(super) fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|left, right| {
        compare_versions(left, right).unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
}

use composer::composer_release_versions_for_package;
use cran::cran_release_versions;
pub use dispatch::{
    LatestVersionRequest, latest_version_from_response, latest_version_from_response_for_request,
    latest_version_from_response_with_prereleases,
};
use docker::docker_build_versions;
pub use docker::docker_tag_exists;
use dotnet::dotnet_release_versions;
pub use endpoint::{
    latest_version_from_response_for_endpoint, release_versions_from_response_for_endpoint,
};
pub use errors::{
    RegistryErrorStatus, http_status_message_from_code, npm_error_status_from_response,
};
use github::is_commit_sha;
use hex::hex_release_versions;
pub use npm::{npm_build_versions, npm_release_versions};
use python::python_release_versions;
use ruby::ruby_release_versions;
use xml::maven_release_versions;

/// Lists the build variants of a version found in a response body.
pub fn build_versions_from_response(
    ecosystem: Ecosystem,
    body: &str,
    requirement: &str,
) -> Vec<String> {
    match ecosystem {
        Docker => docker_build_versions(body, requirement),
        Npm => npm_build_versions(body, requirement),
        _ => vec![],
    }
}

/// Lists the release versions found in a response body.
pub fn release_versions_from_response(ecosystem: Ecosystem, body: &str) -> Vec<String> {
    if matches!(ecosystem, Composer | Cran) {
        return vec![];
    }

    release_versions_from_response_for_package(ecosystem, "", body)
}

/// Extract release versions with the package identity required by registries
/// whose response bodies can contain multiple packages.
pub fn release_versions_from_response_for_package(
    ecosystem: Ecosystem,
    package: &str,
    body: &str,
) -> Vec<String> {
    match ecosystem {
        Composer => composer_release_versions_for_package(body, package),
        Cran => cran_release_versions(body, package),
        Dotnet => dotnet_release_versions(body),
        Hex => hex_release_versions(body),
        Maven => maven_release_versions(body),
        Npm => npm_release_versions(body),
        GitHub => github_release_versions(body),
        Python => python_release_versions(body),
        Ruby => ruby_release_versions(body),
        _ => vec![],
    }
}

fn github_release_versions(body: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(body) else {
        return vec![];
    };
    let Some(tags) = value.as_array() else {
        return vec![];
    };
    let mut versions = tags
        .iter()
        .filter_map(|entry| entry.as_str().or_else(|| entry.get("name")?.as_str()))
        .filter(|tag| !tag.is_empty() && !is_commit_sha(tag))
        .map(|tag| tag.trim_start_matches(['v', 'V']).to_owned())
        .collect::<Vec<_>>();
    versions.sort_by(|left, right| {
        versionlens_model::normalized_version(left)
            .cmp(&versionlens_model::normalized_version(right))
    });
    versions.dedup();
    versions
}

#[cfg(test)]
mod tests {
    use versionlens_model::Ecosystem;

    use crate::{
        vulnerability_advisories_from_response, vulnerability_request_body,
        vulnerability_titles_from_response, vulnerability_url,
        vulnerability_version_from_requirement,
    };

    use super::{
        LatestVersionRequest, build_versions_from_response, docker_tag_exists,
        http_status_message_from_code, latest_version_from_response,
        latest_version_from_response_for_request, latest_version_from_response_with_prereleases,
        npm_build_versions, npm_error_status_from_response, release_versions_from_response,
        release_versions_from_response_for_package,
    };

    fn latest_version_for_requirement(
        ecosystem: Ecosystem,
        package: &str,
        requirement: &str,
        body: &str,
    ) -> Option<String> {
        latest_version_from_response_for_request(LatestVersionRequest {
            ecosystem,
            package,
            requirement,
            body,
            include_prereleases: false,
            prerelease_tags: &[],
        })
    }

    fn latest_version_with_tags(
        ecosystem: Ecosystem,
        package: &str,
        body: &str,
        prerelease_tags: &[String],
    ) -> Option<String> {
        latest_version_from_response_for_request(LatestVersionRequest {
            ecosystem,
            package,
            requirement: "",
            body,
            include_prereleases: true,
            prerelease_tags,
        })
    }

    fn assert_latest(ecosystem: Ecosystem, package: &str, body: &str, expected: &str) {
        assert_eq!(
            latest_version_from_response(ecosystem, package, body),
            Some(expected.to_owned())
        );
    }

    fn osv_range_response(id: &str, summary: &str, introduced: &str, fixed: &str) -> String {
        serde_json::json!({
            "vulns": [{
                "id": id,
                "summary": summary,
                "affected": [{
                    "package": {"name": "left-pad"},
                    "ranges": [{"events": [{"introduced": introduced}, {"fixed": fixed}]}]
                }]
            }]
        })
        .to_string()
    }

    mod composer {
        use super::{
            latest_version_from_response, release_versions_from_response,
            release_versions_from_response_for_package,
        };
        use versionlens_model::Ecosystem::Composer;

        #[test]
        fn picks_latest_stable_composer_version() {
            assert_eq!(
                latest_version_from_response(
                    Composer,
                    "phpunit/phpunit",
                    r#"{"packages":{"phpunit/phpunit":[{"version":"v1.9"},{"version":"10.0.0"},{"version":"11.0.0-beta.1"},{"version":"10.5.0"}]}}"#,
                ),
                Some("10.5.0".to_owned())
            );
        }

        #[test]
        fn picks_latest_composer_version_from_metadata_keys() {
            assert_eq!(
                latest_version_from_response(
                    Composer,
                    "phpunit/phpunit",
                    r#"{"packages":{"phpunit/phpunit":{"10.0.0":{},"11.0.0-beta.1":{},"10.5.0":{}}}}"#,
                ),
                Some("10.5.0".to_owned())
            );
        }

        #[test]
        fn normalizes_v_prefixed_packagist_versions() {
            assert_eq!(
                latest_version_from_response(
                    Composer,
                    "php-parallel-lint/php-parallel-lint",
                    r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v3.1.3"},{"version":"v3.0"}]}}"#,
                ),
                Some("3.1.3".to_owned())
            );
        }

        #[test]
        fn extracts_composer_release_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Composer,
                    "php-parallel-lint/php-parallel-lint",
                    r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v3.1.3"},{"version":"v3.1.2"},{"version":"v3.1.1"},{"version":"v3.1.0"},{"version":"v3.2.0-beta.1"},{"version":"v3.0.1"},{"version":"v3.0"}]}}"#,
                ),
                vec![
                    "3.0.0".to_owned(),
                    "3.0.1".to_owned(),
                    "3.1.0".to_owned(),
                    "3.1.1".to_owned(),
                    "3.1.2".to_owned(),
                    "3.1.3".to_owned(),
                    "3.2.0-beta.1".to_owned(),
                ]
            );
        }

        #[test]
        fn extracts_composer_releases_only_for_the_requested_package() {
            let body = r#"{
      "packages": {
        "acme/target": [{"version": "1.0.0"}, {"version": "1.1.0"}],
        "acme/unrelated": [{"version": "9.0.0"}]
      }
    }"#;

            assert_eq!(
                release_versions_from_response_for_package(Composer, "acme/target", body),
                vec!["1.0.0".to_owned(), "1.1.0".to_owned()]
            );
            assert_eq!(
                release_versions_from_response_for_package(Composer, "acme/unrelated", body),
                vec!["9.0.0".to_owned()]
            );
            assert_eq!(
                release_versions_from_response(Composer, body),
                [] as [String; 0]
            );
        }

        #[test]
        fn extracts_composer_branch_alias_versions_from_metadata() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Composer,
                    "acme/pkg",
                    r#"{
              "packages": {
                "acme/pkg": [
                  {
                    "version": "dev-main",
                    "extra": {
                      "branch-alias": {
                        "dev-main": "1.0.x-dev"
                      }
                    }
                  },
                  {"version": "1.0.0"}
                ]
              }
            }"#,
                ),
                vec!["1.0.0".to_owned(), "1.0.x-dev".to_owned()]
            );
        }

        #[test]
        fn reads_packagist_json_api_package_versions() {
            let body = r#"{
      "package": {
        "name": "monolog/monolog",
        "versions": {
          "3.7.0": {"version": "3.7.0"},
          "3.8.0-beta.1": {"version": "3.8.0-beta.1"},
          "3.6.0": {"version": "3.6.0"}
        }
      }
    }"#;

            assert_eq!(
                latest_version_from_response(Composer, "monolog/monolog", body),
                Some("3.7.0".to_owned())
            );
            assert_eq!(
                release_versions_from_response_for_package(Composer, "monolog/monolog", body),
                vec![
                    "3.6.0".to_owned(),
                    "3.7.0".to_owned(),
                    "3.8.0-beta.1".to_owned(),
                ]
            );
        }

        #[test]
        fn extracts_packagist_json_api_releases_only_when_the_package_matches() {
            let body = r#"{
      "package": {
        "name": "acme/target",
        "versions": {"1.0.0": {}, "1.1.0": {}}
      }
    }"#;

            assert_eq!(
                release_versions_from_response_for_package(Composer, "acme/target", body),
                vec!["1.0.0".to_owned(), "1.1.0".to_owned()]
            );
            assert_eq!(
                release_versions_from_response_for_package(Composer, "acme/unrelated", body),
                [] as [String; 0]
            );
        }
    }
    mod docker {
        use super::{
            build_versions_from_response, docker_tag_exists, latest_version_for_requirement,
            latest_version_from_response, latest_version_from_response_with_prereleases,
        };
        use versionlens_model::Ecosystem::Docker;

        #[test]
        fn ignores_docker_hub_tags_without_digests() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"24","tag_status":"active"},{"name":"22","tag_status":"active","digest":"sha256-22"}]}"#,
                ),
                Some("22".to_owned())
            );
        }

        #[test]
        fn ignores_docker_hub_results_without_active_tag_status() {
            let body = r#"{"results":[{"name":"2.0.0","digest":"sha256-2"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#;

            assert_eq!(
                latest_version_from_response(Docker, "node", body),
                Some("1.0.0".to_owned())
            );
            assert_eq!(docker_tag_exists(body, "2.0.0"), Some(false));
        }

        #[test]
        fn detects_docker_tag_existence() {
            let body = r#"{"results":[{"name":"2.0.0","tag_status":"active","digest":"sha256-2"},{"name":"old","tag_status":"inactive","digest":"sha256-old"},{"name":"broken","tag_status":"active"}]}"#;

            assert_eq!(docker_tag_exists(body, "2.0.0"), Some(true));
            assert_eq!(docker_tag_exists(body, "3"), Some(false));
            assert_eq!(docker_tag_exists(body, "old"), Some(false));
            assert_eq!(docker_tag_exists(body, "broken"), Some(false));
        }

        #[test]
        fn reads_docker_build_versions_from_same_digest_aliases() {
            assert_same_digest_build_versions("23.11.0");
        }

        #[test]
        fn reads_docker_build_versions_for_empty_requirement_from_latest_alias() {
            assert_same_digest_build_versions("");
        }

        fn assert_same_digest_build_versions(requirement: &str) {
            assert_eq!(
                build_versions_from_response(
                    Docker,
                    &versionlens_test_support::fixture!(
                        "tests/fixtures/shared/docker",
                        "node-same-digest.json"
                    )
                    .expect("shared Docker response fixture must be readable"),
                    requirement,
                ),
                serde_json::from_str::<Vec<String>>(
                    &versionlens_test_support::fixture!(
                        "tests/fixtures/shared/docker",
                        "node-same-digest-builds.json"
                    )
                    .expect("shared Docker build fixture must be readable"),
                )
                .expect("shared Docker build fixture must be valid")
            );
        }

        #[test]
        fn reads_docker_latest_build_family_aliases() {
            assert_eq!(
                build_versions_from_response(
                    Docker,
                    r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#,
                    "latest",
                ),
                [
                    "latest".to_owned(),
                    "2022-RTM-CU2-ubuntu-20.04".to_owned(),
                    "2022-RTM-GDR1-ubuntu-20.04".to_owned(),
                    "2022-RTM-ubuntu-20.04".to_owned(),
                    "2022-latest".to_owned(),
                    "2022-preview-ubuntu-22.04".to_owned(),
                ]
            );
        }

        #[test]
        fn does_not_expand_docker_latest_builds_to_older_numeric_family_tags() {
            assert_eq!(
                build_versions_from_response(
                    Docker,
                    r#"{"results":[{"name":"23.10.0","tag_status":"active","digest":"sha256-old"},{"name":"23.11.0","tag_status":"active","digest":"sha256-latest"},{"name":"latest","tag_status":"active","digest":"sha256-latest"}]}"#,
                    "latest",
                ),
                ["latest".to_owned(), "23.11.0".to_owned()]
            );
        }

        #[test]
        fn orphan_docker_hub_latest_tag_falls_back_to_highest_versioned_tag() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-22"},{"name":"24","tag_status":"active","digest":"sha256-24"}]}"#,
                ),
                Some("24".to_owned())
            );
        }

        #[test]
        fn docker_latest_alias_without_version_digest_uses_highest_versioned_tag() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#
                ),
                Some("1.0.0".to_owned())
            );
        }

        #[test]
        fn docker_ignores_five_digit_numeric_tags_like_upstream() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"20240501","tag_status":"active","digest":"sha256-date"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#
                ),
                Some("1.0.0".to_owned())
            );
        }

        #[test]
        fn ignores_docker_four_segment_numeric_tags_like_upstream() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"1.2.3.4","tag_status":"active","digest":"sha256-four"},{"name":"1.2.3","tag_status":"active","digest":"sha256-three"}]}"#
                ),
                Some("1.2.3".to_owned())
            );
        }

        #[test]
        fn normalizes_docker_numeric_tags_with_leading_zeroes_like_upstream() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"01.02.003","tag_status":"active","digest":"sha256-leading"}]}"#
                ),
                Some("1.2.3".to_owned())
            );
        }

        #[test]
        fn reads_docker_numeric_range_alias_builds_across_digests() {
            assert_eq!(
                build_versions_from_response(
                    Docker,
                    r#"{"results":[{"name":"1","tag_status":"active","digest":"sha256-major"},{"name":"1.0","tag_status":"active","digest":"sha256-minor"},{"name":"1.0.0","tag_status":"active","digest":"sha256-fixed"}]}"#,
                    "1.0.0",
                ),
                ["1".to_owned(), "1.0".to_owned(), "1.0.0".to_owned()]
            );
        }

        #[test]
        fn resolves_docker_latest_tag_to_semver_alias() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"22","tag_status":"active","digest":"sha256-22"}]}"#
                ),
                Some("23.11.0".to_owned())
            );
        }

        #[test]
        fn reads_docker_build_suffix_tags() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"24.1.0-bookworm","tag_status":"active","digest":"sha256-24"}]}"#,
                ),
                Some("24.1.0-bookworm".to_owned())
            );
        }

        #[test]
        fn keeps_docker_build_suffix_when_matching_tag_exists() {
            assert_eq!(
                latest_version_for_requirement(
                    Docker,
                    "node",
                    "20.19.1-bookworm",
                    r#"{"results":[{"name":"25.0.0-alpine","tag_status":"active","digest":"sha256-25"},{"name":"24.1.0-bookworm","tag_status":"active","digest":"sha256-24"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"}]}"#,
                ),
                Some("24.1.0-bookworm".to_owned())
            );
        }

        #[test]
        fn filters_docker_dotted_prerelease_tags() {
            let body = r#"{"results":[{"name":"24.1.0","tag_status":"active","digest":"sha256-24"},{"name":"25.0.0-rc.1","tag_status":"active","digest":"sha256-25"}]}"#;

            assert_eq!(
                latest_version_from_response(Docker, "node", body),
                Some("24.1.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(Docker, "node", body, true),
                Some("25.0.0-rc.1".to_owned())
            );
        }

        #[test]
        fn reads_microsoft_container_registry_tags() {
            assert_eq!(
                latest_version_from_response(
                    Docker,
                    "mcr.microsoft.com/dotnet/sdk",
                    r#"[{"name":"8.0","digest":"sha256-8"},{"name":"9.0-preview","digest":"sha256-9"},{"name":"10.0","digest":"sha256-10"}]"#
                ),
                Some("10.0".to_owned())
            );
        }

        #[test]
        fn reads_docker_registry_v2_tag_lists() {
            let body = r#"{"name":"org/app","tags":["1.0.0","1.4.0","2.0.0-rc.1"]}"#;

            assert_eq!(
                latest_version_from_response(Docker, "ghcr.io/org/app", body),
                Some("1.4.0".to_owned())
            );
            assert_eq!(docker_tag_exists(body, "1.4.0"), Some(true));
            assert_eq!(docker_tag_exists(body, "3.0.0"), Some(false));
        }
    }
    mod dotnet {
        use super::release_versions_from_response_for_package;
        use versionlens_model::Ecosystem::Dotnet;

        #[test]
        fn extracts_dotnet_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Dotnet,
                    "Example.Package",
                    r#"{"versions":["1.4.1","1.4.2","1.4.3-beta.1","1.5.0","invalid","1.6.2"]}"#,
                ),
                vec![
                    "1.4.1".to_owned(),
                    "1.4.2".to_owned(),
                    "1.4.3-beta.1".to_owned(),
                    "1.5.0".to_owned(),
                    "1.6.2".to_owned(),
                ]
            );
        }
    }
    mod endpoint {
        use crate::{
            LatestVersionRequest, RegistryEndpoint, RegistryResponseKind,
            latest_version_from_response_for_endpoint, release_versions_from_response_for_endpoint,
        };
        use versionlens_model::Ecosystem::*;

        fn latest(endpoint: &RegistryEndpoint, package: &str, body: &str) -> Option<String> {
            latest_version_from_response_for_endpoint(
                endpoint,
                LatestVersionRequest {
                    ecosystem: match endpoint.response_kind {
                        RegistryResponseKind::GoModuleList
                        | RegistryResponseKind::GoModuleLatest => Go,
                        _ => Python,
                    },
                    package,
                    requirement: "",
                    body,
                    include_prereleases: false,
                    prerelease_tags: &[],
                },
            )
        }

        #[test]
        fn python_simple_html_responses_use_project_distribution_files() {
            let endpoint = RegistryEndpoint::new(
                "https://packages.test/simple/my-package/".to_owned(),
                RegistryResponseKind::PythonSimple,
            );
            let body = r#"<!doctype html><html><body>
      <a href="../../files/My_Package-1.0.0.tar.gz">My_Package-1.0.0.tar.gz</a>
      <a href="../../files/my_package-1.1.0-py3-none-any.whl">my_package-1.1.0-py3-none-any.whl</a>
      <a data-yanked="broken" href="../../files/my_package-9.0.0.tar.gz">my_package-9.0.0.tar.gz</a>
      <a href="../../files/unrelated-8.0.0.tar.gz">unrelated-8.0.0.tar.gz</a>
    </body></html>"#;

            assert_eq!(
                latest(&endpoint, "My.Package", body),
                Some("1.1.0".to_owned())
            );
            assert_eq!(
                release_versions_from_response_for_endpoint(&endpoint, Python, "My.Package", body),
                ["1.0.0".to_owned(), "1.1.0".to_owned()]
            );
        }

        #[test]
        fn python_simple_json_responses_filter_yanked_files() {
            let endpoint = RegistryEndpoint::new(
                "https://packages.test/simple/demo/".to_owned(),
                RegistryResponseKind::PythonSimple,
            );
            let body = r#"{"meta":{"api-version":"1.4"},"name":"demo","files":[{"filename":"demo-1.0.0.tar.gz","yanked":false},{"filename":"demo-1.2.0-py3-none-any.whl"},{"filename":"demo-3.0.0.tar.gz","yanked":"broken"}]}"#;

            assert_eq!(latest(&endpoint, "demo", body), Some("1.2.0".to_owned()));
        }

        #[test]
        fn endpoint_parsers_reject_incompatible_response_shapes() {
            let go_list = RegistryEndpoint::new(
                "https://proxy.test/example.com/mod/@v/list".to_owned(),
                RegistryResponseKind::GoModuleList,
            );
            let go_latest = RegistryEndpoint::new(
                "https://proxy.test/example.com/mod/@latest".to_owned(),
                RegistryResponseKind::GoModuleLatest,
            );
            let python_simple = RegistryEndpoint::new(
                "https://packages.test/simple/demo/".to_owned(),
                RegistryResponseKind::PythonSimple,
            );
            let python_rss = RegistryEndpoint::new(
                "https://pypi.org/rss/project/demo/releases.xml".to_owned(),
                RegistryResponseKind::PythonRss,
            );

            assert_eq!(
                latest(&go_list, "example.com/mod", r#"{"Version":"v1.2.0"}"#),
                None
            );
            assert_eq!(
                latest(&go_latest, "example.com/mod", "v1.1.0\nv1.2.0\n"),
                None
            );
            assert_eq!(
                latest(
                    &python_simple,
                    "demo",
                    r#"{"info":{"version":"9.0.0"},"releases":{}}"#,
                ),
                None
            );
            assert_eq!(
                latest(
                    &python_rss,
                    "demo",
                    "<!doctype html><html><a href='demo-1.0.0.tar.gz'>demo-1.0.0.tar.gz</a></html>",
                ),
                None
            );
        }
    }
    mod npm_errors {
        use super::{
            http_status_message_from_code, latest_version_for_requirement,
            npm_error_status_from_response,
        };
        use crate::response::RegistryErrorStatus::{
            Error as RegistryStatusError, Invalid as RegistryStatusInvalid,
            InvalidWithLatest as RegistryStatusInvalidWithLatest,
            NotSupported as RegistryStatusNotSupported,
        };
        use versionlens_model::Ecosystem::Npm;

        #[test]
        fn resolves_npm_dist_tag_requirements() {
            assert_eq!(
                latest_version_for_requirement(
                    Npm,
                    "typescript",
                    "next",
                    r#"{"dist-tags":{"latest":"6.0.3","next":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
                ),
                Some("7.0.0-beta.1".to_owned())
            );
        }

        #[test]
        fn reads_npm_error_statuses_from_responses() {
            assert_eq!(
                npm_error_status_from_response(r#"{"status":"ECONNREFUSED"}"#),
                Some(RegistryStatusError("connection refused".to_owned()))
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"code":"EUNSUPPORTEDPROTOCOL"}"#),
                Some(RegistryStatusNotSupported)
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"error":{"code":"EINVALIDTAGNAME"}}"#),
                Some(RegistryStatusInvalidWithLatest(
                    "invalid version".to_owned()
                ))
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"error":{"code":"EINVALIDPACKAGENAME"}}"#),
                Some(RegistryStatusInvalid("invalid version".to_owned()))
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"status":"E404"}"#),
                Some(RegistryStatusError("not found".to_owned()))
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"status":128}"#),
                Some(RegistryStatusError("not found".to_owned()))
            );
            assert_eq!(
                npm_error_status_from_response(r#"{"status":"ENOTFOUND"}"#),
                Some(RegistryStatusError("ENOTFOUND".to_owned()))
            );
        }

        #[test]
        fn maps_known_http_status_messages() {
            assert_eq!(http_status_message_from_code(400), Some("400 bad request"));
            assert_eq!(
                http_status_message_from_code(401),
                Some("401 not authorized")
            );
            assert_eq!(http_status_message_from_code(403), Some("403 forbidden"));
            assert_eq!(http_status_message_from_code(404), Some("not found"));
            assert_eq!(
                http_status_message_from_code(500),
                Some("500 internal server error")
            );
            assert_eq!(http_status_message_from_code(418), None);
        }
    }
    mod versions {
        use super::{
            assert_latest, latest_version_for_requirement, latest_version_from_response,
            latest_version_from_response_with_prereleases, latest_version_with_tags,
            npm_build_versions, release_versions_from_response,
            release_versions_from_response_for_package,
        };
        use versionlens_model::Ecosystem::*;

        #[test]
        #[expect(
            clippy::too_many_lines,
            reason = "table-driven manifest coverage stays readable as one scenario"
        )]
        fn reads_latest_versions_from_json_registry_responses() {
            for (ecosystem, package, body, expected) in [
                (
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"1.0.228"}}"#,
                    "1.0.228",
                ),
                (
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"1.0.229"},"versions":[{"num":"1.0.229","yanked":true},{"num":"1.0.228","yanked":false}]}"#,
                    "1.0.228",
                ),
                (
                    Npm,
                    "typescript",
                    r#"{"dist-tags":{"latest":"6.0.3"}}"#,
                    "6.0.3",
                ),
                (
                    Npm,
                    "octokit/core.js",
                    r#"[{"name":"v2.5.0"},{"name":"v2.0.0"}]"#,
                    "v2.5.0",
                ),
                (
                    Deno,
                    "@std/assert",
                    r#"{"versions":{"1.0.0":{"yanked":true},"1.1.0":{},"1.2.0-rc.1":{},"1.0.3":{}}}"#,
                    "1.1.0",
                ),
                (
                    Deno,
                    "@std/assert",
                    r#"{"latest":"1.3.0","versions":{"1.2.0":{},"1.3.0":{}}}"#,
                    "1.3.0",
                ),
                (
                    Dotnet,
                    "Newtonsoft.Json",
                    r#"{"versions":["13.0.1","14.0.0-beta.1","13.0.3"]}"#,
                    "13.0.3",
                ),
                (
                    Docker,
                    "node",
                    r#"{"results":[{"name":"20","tag_status":"active","digest":"sha256-20"},{"name":"22-beta","tag_status":"active","digest":"sha256-22"},{"name":"18","tag_status":"inactive","digest":"sha256-18"}]}"#,
                    "20",
                ),
                (Python, "flask", r#"{"info":{"version":"3.0.0"}}"#, "3.0.0"),
                (
                    Python,
                    "flask",
                    r#"{"info":{"version":"3.1.0"},"releases":{"3.0.0":[{"yanked":false}],"3.1.0":[{"yanked":true}]}}"#,
                    "3.0.0",
                ),
                (
                    Pub,
                    "http",
                    r#"{"versions":[{"version":"1.0.0"},{"version":"1.2.0","retracted":true},{"version":"1.1.0"},{"version":"2.0.0-dev.1"}]}"#,
                    "1.1.0",
                ),
                (
                    Ruby,
                    "rails",
                    r#"[{"number":"8.0.0"},{"number":"8.1.0.beta1"},{"number":"8.0.4"}]"#,
                    "8.0.4",
                ),
                (
                    Ruby,
                    "rspec/rspec-rails",
                    r#"[{"name":"v6.1.0"},{"name":"v6.0.1"}]"#,
                    "v6.1.0",
                ),
                (
                    Vcpkg,
                    "fmt",
                    r#"{"versions":[{"version":"11.1.4","git-tree":"a"},{"version":"11.2.0-rc.1","git-tree":"b"},{"version":"10.2.1#1","git-tree":"c"}]}"#,
                    "11.1.4",
                ),
                (
                    Terraform,
                    "hashicorp/aws",
                    r#"{"versions":[{"version":"5.0.0"},{"version":"5.1.0-beta.1"},{"version":"4.67.0"}]}"#,
                    "5.0.0",
                ),
                (
                    AnsibleGalaxy,
                    "community.general",
                    r#"{"data":[{"version":"8.0.0-beta.1"},{"version":"7.5.0"}]}"#,
                    "7.5.0",
                ),
                (
                    Bazel,
                    "rules_cc",
                    r#"{"versions":["0.0.9","0.0.10-rc1","0.0.10"],"yanked_versions":{"0.0.10":"bad release"}}"#,
                    "0.0.9",
                ),
                (
                    Nix,
                    "NixOS/nixpkgs",
                    r#"[{"name":"nixos-24.05"},{"name":"nixos-unstable"},{"name":"23.11"}]"#,
                    "24.05",
                ),
                (
                    CocoaPods,
                    "AFNetworking",
                    r#"{"versions":[{"name":"4.0.1"},{"name":"4.0.0-beta.1"},{"name":"3.2.1"}]}"#,
                    "4.0.1",
                ),
            ] {
                assert_latest(ecosystem, package, body, expected);
            }
        }

        #[test]
        fn reads_full_crates_io_response_without_deserializing_unneeded_fields() {
            assert_latest(
                Cargo,
                "serde",
                r#"{
            "crate": {
                "id": "serde",
                "name": "serde",
                "downloads": 900000000,
                "max_version": "2.0.0-alpha.1",
                "max_stable_version": "1.0.228",
                "description": "serialization",
                "repository": "https://github.com/serde-rs/serde"
            },
            "versions": [
                {"id": 1, "crate": "serde", "num": "1.0.229", "downloads": 1, "yanked": true, "features": {}},
                {"id": 2, "crate": "serde", "num": "1.0.228", "downloads": 1, "yanked": false, "features": {}}
            ],
            "keywords": [{"id": "serialization"}],
            "categories": [{"id": "encoding"}]
        }"#,
                "1.0.228",
            );
        }

        #[test]
        fn reads_abbreviated_npm_metadata_without_materializing_version_payloads() {
            assert_latest(
                Npm,
                "react",
                r#"{
            "dist-tags":{"latest":"19.1.1"},
            "versions":{
                "18.3.1":{"dependencies":{"loose-envify":"^1.1.0"}},
                "19.1.1":{"dist":{"tarball":"https://registry.example/react.tgz"}}
            },
            "readme":"a decoy \"dist-tags\" and \"versions\""
        }"#,
                "19.1.1",
            );
        }

        #[test]
        fn rejects_malformed_cargo_responses() {
            for body in [
                "not json",
                r#"{"versions":"1.0.0"}"#,
                r#"{"versions":[{"num":1,"yanked":false}]}"#,
            ] {
                assert_eq!(latest_version_from_response(Cargo, "serde", body), None);
            }
        }

        #[test]
        fn reads_only_a_valid_included_crates_io_default_version() {
            assert_latest(
                Cargo,
                "serde",
                r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[{"num":"1.0.228","yanked":false}]}"#,
                "1.0.228",
            );
            for body in [
                r#"{"crate":{"default_version":"2.0.0-alpha.1","max_version":"0.0.0"},"versions":[{"num":"2.0.0-alpha.1","yanked":false}]}"#,
                r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[{"num":"1.0.228","yanked":true}]}"#,
                r#"{"crate":{"default_version":null,"max_version":"0.0.0","yanked":true},"versions":[]}"#,
                r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[]}"#,
            ] {
                assert_eq!(latest_version_from_response(Cargo, "serde", body), None);
            }
        }

        #[test]
        fn reads_cargo_prerelease_pages_and_metadata() {
            let body = r#"{
        "versions":[
            {"num":"2.0.0-rc.1","yanked":true},
            {"num":"2.0.0-beta.2","yanked":false},
            {"num":"1.9.0","yanked":false}
        ],
        "meta":{"next_page":"?per_page=100&seek=next"}
    }"#;
            assert_eq!(
                latest_version_with_tags(Cargo, "serde", body, &["beta".to_owned()]),
                Some("2.0.0-beta.2".to_owned())
            );
            assert_eq!(
                latest_version_with_tags(Cargo, "serde", body, &["rc".to_owned()]),
                Some("1.9.0".to_owned())
            );
        }
        #[test]
        fn github_commit_arrays_follow_the_pinned_sha_length() {
            let body = r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef"}]"#;
            for ecosystem in [Npm, Ruby] {
                for (requirement, expected) in [
                    ("1234567", Some("abcdef1")),
                    (
                        "1234567890123456789012345678901234567890",
                        Some("abcdef1234567890abcdef1234567890abcdef12"),
                    ),
                    ("main", None),
                    ("", None),
                ] {
                    assert_eq!(
                        latest_version_for_requirement(
                            ecosystem,
                            "owner/commit",
                            requirement,
                            body
                        ),
                        expected.map(str::to_owned)
                    );
                }
            }
        }

        #[test]
        fn reads_github_action_tags_and_release_history() {
            let body = r#"[{"name":"v0.7.0"},{"name":"v0.6.1"},{"name":"v0.6.0"}]"#;
            assert_latest(GitHub, "actions/checkout", body, "v0.7.0");
            assert_latest(
                GitHub,
                "acme/automation",
                r#"[{"name":"v1.3.0"},{"name":"v1"}]"#,
                "v1.3.0",
            );
            assert_eq!(
                release_versions_from_response_for_package(GitHub, "actions/checkout", body),
                ["0.6.0".to_owned(), "0.6.1".to_owned(), "0.7.0".to_owned()]
            );
        }

        #[test]
        fn github_action_tag_responses_ignore_commit_sha_tags() {
            let body =
                r#"[{"name":"vabcdef1234567890abcdef1234567890abcdef12"},{"name":"v4.2.0"}]"#;

            assert_latest(GitHub, "actions/checkout", body, "v4.2.0");
            assert_eq!(
                release_versions_from_response_for_package(GitHub, "actions/checkout", body),
                ["4.2.0".to_owned()]
            );
        }

        #[test]
        fn github_tag_responses_ignore_non_semver_names() {
            for ecosystem in [Npm, Ruby] {
                assert_latest(
                    ecosystem,
                    "owner/repo",
                    r#"[{"name":"release-5.6.8"},{"name":"v2.0.0"},{"name":"build-9.0.0"}]"#,
                    "v2.0.0",
                );
            }
        }

        #[test]
        fn cpp_github_tag_responses_use_json_before_xmake_text_fallback() {
            assert_latest(
                Cpp,
                "fmtlib/fmt",
                r#"[{"name":"11.1.4"},{"name":"11.2.0-rc.1"},{"name":"10.2.1"}]"#,
                "11.1.4",
            );
            assert_latest(
                Cpp,
                "openssl",
                r#"add_versions("3.0.0", "sha256") add_versions("3.1.0-beta", "sha256")"#,
                "3.0.0",
            );
        }

        #[test]
        fn reads_ruby_versions_from_normalized_string_arrays() {
            assert_latest(Ruby, "rails", r#"["1.0.0","1.1.0","2.0.0-alpha"]"#, "1.1.0");
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Ruby,
                    "rails",
                    r#"["1.0.0","1.1.0","2.0.0-alpha"]"#,
                    true,
                ),
                Some("2.0.0-alpha".to_owned())
            );
        }

        #[test]
        fn reads_deno_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Deno,
                    "@std/assert",
                    r#"["0.215.0","0.212.0","1.0.6","0.198.0","0.196.0","1.1.0-rc.2"]"#,
                ),
                Some("1.0.6".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Deno,
                    "@std/assert",
                    r#"["0.215.0","0.212.0","1.0.6","0.198.0","0.196.0","1.1.0-rc.2"]"#,
                    true,
                ),
                Some("1.1.0-rc.2".to_owned())
            );
        }

        #[test]
        fn reads_cargo_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Cargo,
                    "serde",
                    r#"{"versions":["1.0.20","1.0.19","1.1.0-beta.1"]}"#,
                ),
                Some("1.0.20".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Cargo,
                    "serde",
                    r#"["1.0.20","1.0.19","1.1.0-beta.1"]"#,
                    true,
                ),
                Some("1.1.0-beta.1".to_owned())
            );
        }

        #[test]
        fn reads_hex_versions_from_package_releases() {
            assert_eq!(
                latest_version_from_response(
                    Hex,
                    "plug",
                    r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
                ),
                Some("1.20.2".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Hex,
                    "plug",
                    r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
                    true,
                ),
                Some("1.21.0-rc.1".to_owned())
            );
        }

        #[test]
        fn ignores_malformed_hex_package_responses() {
            assert_eq!(
                latest_version_from_response(Hex, "plug", r#"{"releases": ["#),
                None
            );
            assert_eq!(
                latest_version_from_response(Hex, "plug", r#"{"package":"plug"}"#),
                None
            );
            assert_eq!(
                release_versions_from_response_for_package(Hex, "plug", r#"{"releases": ["#),
                [] as [String; 0]
            );
        }

        #[test]
        fn extracts_hex_release_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Hex,
                    "plug",
                    r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
                ),
                [
                    "1.19.4".to_owned(),
                    "1.20.2".to_owned(),
                    "1.21.0-rc.1".to_owned()
                ]
            );
        }

        #[test]
        fn reads_latest_opam_version_from_package_page() {
            assert_eq!(
                latest_version_from_response(
                    Opam,
                    "lwt",
                    r"<h2>lwt version</h2><p>6.1.2 (latest)</p>",
                ),
                Some("6.1.2".to_owned())
            );
        }

        #[test]
        fn reads_latest_hackage_version_from_package_versions() {
            assert_eq!(
                latest_version_from_response(
                    Hackage,
                    "aeson",
                    r#"{"2.1.2.1":"normal","2.2.3.0":"normal","2.2.4.0":"deprecated","2.3.0.0-rc1":"normal"}"#,
                ),
                Some("2.2.3.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Hackage,
                    "aeson",
                    r#"{"2.1.2.1":"normal","2.2.3.0":"normal","2.3.0.0-rc1":"normal"}"#,
                    true,
                ),
                Some("2.3.0.0-rc1".to_owned())
            );
        }

        #[test]
        fn reads_latest_stackage_lts_snapshot_from_snapshot_index() {
            assert_eq!(
                latest_version_from_response(
                    Hackage,
                    "stackage-lts",
                    r#"{"snapshots":[[["nightly-2026-07-03","Stackage Nightly 2026-07-03 (ghc-9.12.4)","2 days ago"]],[["lts-24.48","LTS Haskell 24.48 (ghc-9.10.3)","5 days ago"],["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#,
                ),
                Some("24.49".to_owned())
            );
        }

        #[test]
        fn reads_latest_stackage_nightly_snapshot_from_snapshot_index() {
            assert_eq!(
                latest_version_from_response(
                    Hackage,
                    "stackage-nightly",
                    r#"{"snapshots":[[["nightly-2026-07-02","Stackage Nightly 2026-07-02 (ghc-9.12.4)","3 days ago"]],[["nightly-2026-07-03","Stackage Nightly 2026-07-03 (ghc-9.12.4)","2 days ago"]],[["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#,
                ),
                Some("2026-07-03".to_owned())
            );
        }

        #[test]
        fn reads_latest_conan_version_from_search_results() {
            assert_eq!(
                latest_version_from_response(
                    Conan,
                    "zlib",
                    r#"{"results":["zlib/1.2.13","zlib/1.3.1","zlib/1.3.0#recipe"]}"#,
                ),
                Some("1.3.1".to_owned())
            );
        }

        #[test]
        fn reads_latest_vcpkg_version_from_versions_database_entry() {
            assert_eq!(
                latest_version_from_response(
                    Vcpkg,
                    "fmt",
                    r#"{"versions":[{"version":"11.1.4","git-tree":"a"},{"version":"11.2.0-rc.1","git-tree":"b"},{"version":"10.2.1#1","git-tree":"c"}]}"#,
                ),
                Some("11.1.4".to_owned())
            );
        }

        #[test]
        fn reads_latest_swift_versions_from_registry_and_github_responses() {
            assert_eq!(
                latest_version_from_response(
                    Swift,
                    "mona.LinkedList",
                    r#"{"releases":{"1.1.0":{"url":"https://packages.example.com/mona/LinkedList/1.1.0"},"1.2.0-beta.1":{"url":"https://packages.example.com/mona/LinkedList/1.2.0-beta.1"},"1.0.0":{"problem":{"status":410}}}}"#,
                ),
                Some("1.1.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Swift,
                    "apple/swift-nio",
                    r#"[{"name":"2.66.0"},{"name":"2.67.0-alpha.1"},{"name":"2.65.0"}]"#,
                ),
                Some("2.66.0".to_owned())
            );
        }

        #[test]
        fn reads_latest_zig_version_from_github_tags() {
            assert_eq!(
                latest_version_from_response(
                    Zig,
                    "ziglibs/known-folders",
                    r#"[{"name":"0.8.0"},{"name":"0.9.0-dev.1"},{"name":"0.7.0"}]"#,
                ),
                Some("0.8.0".to_owned())
            );
        }

        #[test]
        fn reads_latest_nim_version_from_package_list_or_github_tags() {
            assert_eq!(
                latest_version_from_response(
                    Nim,
                    "jester",
                    r#"[{"name":"jester","url":"https://github.com/dom96/jester","versions":["0.5.0","0.6.0-rc.1","0.4.3"]},{"name":"other","versions":["9.0.0"]}]"#,
                ),
                Some("0.5.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Nim,
                    "user/pkg",
                    r#"[{"name":"2.1.0"},{"name":"2.2.0-beta.1"},{"name":"2.0.0"}]"#,
                ),
                Some("2.1.0".to_owned())
            );
        }

        #[test]
        fn reads_latest_luarocks_versions_from_manifest() {
            assert_eq!(
                latest_version_from_response(
                    LuaRocks,
                    "luasocket",
                    r#"repository = {
   ["luasocket"] = {
      ["3.0.0-1"] = { { arch = "rockspec" } },
      ["3.1.0-1"] = { { arch = "src" } },
      ["3.2.0-rc1"] = { { arch = "src" } }
   },
   ["other"] = {
      ["9.0.0-1"] = { { arch = "src" } }
   }
}"#,
                ),
                Some("3.1.0-1".to_owned())
            );
        }

        #[test]
        fn reads_latest_cpan_version_from_metacpan_download_url() {
            assert_eq!(
                latest_version_from_response(
                    Cpan,
                    "Plack",
                    r#"{"status":"latest","version":"1.0054","download_url":"https://cpan.metacpan.org/authors/id/M/MI/MIYAGAWA/Plack-1.0054.tar.gz"}"#,
                ),
                Some("1.0054".to_owned())
            );
        }

        #[test]
        fn extracts_cran_releases_only_for_the_requested_package() {
            let body = "Package: cli\nVersion: 3.6.2\n\nPackage: dplyr\nVersion: 1.1.3\nDepends: R (>= 3.5.0)\n\nPackage: dplyr\nVersion: 1.1.4\n";
            assert_eq!(
                latest_version_from_response(Cran, "dplyr", body),
                Some("1.1.4".to_owned())
            );
            assert_eq!(
                release_versions_from_response_for_package(Cran, "dplyr", body),
                vec!["1.1.3".to_owned(), "1.1.4".to_owned()]
            );
            assert_eq!(
                release_versions_from_response_for_package(Cran, "cli", body),
                vec!["3.6.2".to_owned()]
            );
            assert_eq!(
                release_versions_from_response(Cran, body),
                [] as [String; 0]
            );
        }

        #[test]
        fn release_versions_compatibility_api_does_not_require_a_package() {
            assert_eq!(
                release_versions_from_response(
                    Hex,
                    r#"{"releases":[{"version":"1.20.2"},{"version":"1.19.4"}]}"#,
                ),
                ["1.19.4".to_owned(), "1.20.2".to_owned()]
            );
        }

        #[test]
        fn reads_latest_julia_version_from_registry_versions_toml() {
            assert_eq!(
                latest_version_from_response(
                    Julia,
                    "Example",
                    r#"[0.5.3]
git-tree-sha1 = "b4d4"

[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0-rc1]
git-tree-sha1 = "d6f6"
"#,
                ),
                Some("0.5.4".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Julia,
                    "Example",
                    r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0-rc1]
git-tree-sha1 = "d6f6"
"#,
                    true,
                ),
                Some("0.6.0-rc1".to_owned())
            );
        }
        #[test]
        fn reads_go_versions_from_normalized_version_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "go.uber.org/zap",
                    r#"{"versions":["v0.32.3","v0.19.10","v0.26.0","v0.23.0-alpha.3"]}"#,
                ),
                Some("v0.32.3".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "github.com/docker/cli",
                    r#"["v26.1.3+incompatible","v27.0.0+incompatible"]"#,
                ),
                Some("v27.0.0".to_owned())
            );
        }

        #[test]
        fn reads_go_latest_version_from_proxy_info_object() {
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "golang.org/x/mod",
                    r#"{"Version":"v0.2.0","Time":"2020-01-02T17:33:45Z"}"#,
                ),
                Some("v0.2.0".to_owned())
            );
        }

        #[test]
        fn go_module_proxy_versions_fall_back_to_highest_prerelease_when_no_release_exists() {
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "example.com/mod",
                    r#"{"versions":["v1.0.0-beta.1","v1.0.0-beta.2"]}"#,
                ),
                Some("v1.0.0-beta.2".to_owned())
            );
        }

        #[test]
        fn go_module_proxy_versions_fall_back_to_most_recent_pseudo_version_when_no_release_or_prerelease_exists()
         {
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "example.com/mod",
                    r#"{"versions":["v1.2.4-0.20200101000000-aaaaaaaaaaaa","v1.2.3-0.20240202000000-bbbbbbbbbbbb"]}"#,
                ),
                Some("v1.2.3-0.20240202000000-bbbbbbbbbbbb".to_owned())
            );
        }

        #[test]
        fn go_json_metadata_filters_retracted_and_deprecated_versions_when_available() {
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "example.com/mod",
                    r#"{"versions":[{"Version":"v1.0.0"},{"Version":"v1.1.0","Retracted":["bad release"]},{"Version":"v1.2.0-beta.1","Deprecated":"use example.com/mod/v2"}]}"#,
                ),
                Some("v1.0.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Go,
                    "example.com/mod",
                    r#"{"versions":[{"Version":"v1.0.0"},{"Version":"v1.1.0","Deprecated":"use example.com/mod/v2"}]}"#,
                ),
                Some("v1.0.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Go,
                    "example.com/mod",
                    r#"[{"Version":"v1.0.0"},{"Version":"v1.1.0-beta.1","Retracted":[]},{"Version":"v1.2.0-beta.1","Retracted":null}]"#,
                    true,
                ),
                Some("v1.0.0".to_owned())
            );
        }
        #[test]
        fn reads_python_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Python,
                    "pip",
                    r#"["25.0.1","25.0","24.3.1","24.3","24.2"]"#,
                ),
                Some("25.0.1".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Python,
                    "pip",
                    r#"{"versions":["25.0.1","25.0","26.0.0rc1"]}"#,
                    true,
                ),
                Some("26.0.0rc1".to_owned())
            );
        }

        #[test]
        fn extracts_python_release_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Python,
                    "pip",
                    r#"{"versions":["24.3.1","25.0.0","26.0.0rc1"]}"#,
                ),
                [
                    "24.3.1".to_owned(),
                    "25.0.0".to_owned(),
                    "26.0.0-rc.1".to_owned()
                ]
            );
            assert_eq!(
                release_versions_from_response_for_package(
                    Python,
                    "pip",
                    r#"{"info":{"version":"3.0.0"},"releases":{"2.0.0":[{"yanked":true}],"1.0.0":[],"1.1.0":[{"yanked":false}],"1.0":[{"yanked":false}],"1.1.0rc1":[{"yanked":false}]}}"#,
                ),
                [
                    "1.0.0".to_owned(),
                    "1.1.0-rc.1".to_owned(),
                    "1.1.0".to_owned()
                ]
            );
            assert_eq!(
                release_versions_from_response_for_package(
                    Python,
                    "demo",
                    r#"<?xml version="1.0"?><rss><channel><item><title>Demo 1.0.0</title></item><item><title>Demo 1.1.0</title></item><item><title>Demo 1.1.0</title></item><item><title>Demo 2.0.0rc1</title></item></channel></rss>"#,
                ),
                [
                    "1.0.0".to_owned(),
                    "1.1.0".to_owned(),
                    "2.0.0-rc.1".to_owned()
                ]
            );
        }

        #[test]
        fn extracts_ruby_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Ruby,
                    "rails",
                    r#"["1.0.0","1.1.0-pre.1","1.0.1"]"#,
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.1".to_owned(),
                    "1.1.0-pre.1".to_owned()
                ]
            );
        }

        #[test]
        fn reads_maven_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Maven,
                    "junit:junit",
                    r#"["4.13-rc-1","4.13-rc-2","4.13","4.13.1","4.13.2"]"#,
                ),
                Some("4.13.2".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Maven,
                    "junit:junit",
                    r#"["4.13-rc-1","4.13-rc-2","4.13","4.13.1","4.13.2","4.14.0-rc.1"]"#,
                    true,
                ),
                Some("4.14.0-rc.1".to_owned())
            );
        }

        #[test]
        fn reads_maven_release_versions_for_update_choices() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Maven,
                    "junit:junit",
                    r"<metadata><versioning><versions><version>1.0.0</version><version>2.0.0-M1</version><version>1.0.1</version><version>ignored</version></versions></versioning></metadata>"
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.1".to_owned(),
                    "2.0.0-M1".to_owned()
                ]
            );
        }

        #[test]
        fn reads_npm_release_versions_in_upstream_compare_build_order() {
            assert_eq!(
                release_versions_from_response_for_package(
                    Npm,
                    "example",
                    r#"{"versions":{"2.0.0":{},"1.0.0+build.10":{},"1.0.0+build.2":{},"1.0.0":{}}}"#,
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.0+build.2".to_owned(),
                    "1.0.0+build.10".to_owned(),
                    "2.0.0".to_owned()
                ]
            );
        }

        #[test]
        fn reads_npm_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Npm,
                    "npm-package-arg",
                    r#"{"dist-tags":{"latest":"7.0.0"},"versions":["6.0.0","6.1.0","7.0.0","8.0.0","8.0.1"]}"#,
                ),
                Some("7.0.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Npm,
                    "pacote",
                    r#"{"dist-tags":{"latest":"11.1.9"},"versions":["11.1.9","12.0.0-beta.1"]}"#,
                    true,
                ),
                Some("11.1.9".to_owned())
            );
        }

        #[test]
        fn reads_npm_build_versions_for_matching_release() {
            assert_eq!(
                npm_build_versions(
                    r#"{"versions":{"1.0.0":{},"1.0.0+build.1":{},"1.0.0+build.2":{},"1.1.0+build.1":{}}}"#,
                    "1.0.0+build.1",
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.0+build.1".to_owned(),
                    "1.0.0+build.2".to_owned()
                ]
            );
        }

        #[test]
        fn reads_npm_build_versions_from_normalized_string_arrays() {
            assert_eq!(
                npm_build_versions(
                    r#"{"versions":["1.0.0","1.0.0+build.1","1.0.0+build.2","1.1.0+build.1"]}"#,
                    "1.0.0+build.1",
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.0+build.1".to_owned(),
                    "1.0.0+build.2".to_owned()
                ]
            );
        }
        #[test]
        fn reads_npm_build_versions_in_upstream_compare_build_order() {
            assert_eq!(
                npm_build_versions(
                    r#"{"versions":{"1.0.0+build.10":{},"1.0.0+build.2":{},"1.0.0":{},"1.1.0+build.1":{}}}"#,
                    "1.0.0+build.2",
                ),
                [
                    "1.0.0".to_owned(),
                    "1.0.0+build.2".to_owned(),
                    "1.0.0+build.10".to_owned()
                ]
            );
        }

        #[test]
        fn pub_latest_field_is_used_when_version_list_has_no_allowed_release() {
            assert_eq!(
                latest_version_from_response(
                    Pub,
                    "http",
                    r#"{"latest":{"version":"1.2.3"},"versions":[]}"#,
                ),
                Some("1.2.3".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Pub,
                    "http",
                    r#"{"latest":{"version":"1.2.3"},"versions":[{"version":"2.0.0-dev.1"}]}"#,
                ),
                Some("1.2.3".to_owned())
            );
        }

        #[test]
        fn reads_pub_versions_from_normalized_string_arrays() {
            assert_eq!(
                latest_version_from_response(
                    Pub,
                    "http",
                    r#"{"versions":["0.1.0","0.1.1","0.1.2","1.0.0-dev.1"]}"#,
                ),
                Some("0.1.2".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Pub,
                    "http",
                    r#"{"versions":["0.1.0","0.1.1","0.1.2","1.0.0-dev.1"]}"#,
                    true,
                ),
                Some("1.0.0-dev.1".to_owned())
            );
        }

        #[test]
        fn cargo_crate_metadata_prefers_max_stable_version() {
            assert_eq!(
                latest_version_from_response(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"}}"#,
                ),
                Some("1.0.228".to_owned())
            );
        }

        #[test]
        fn cargo_crate_metadata_is_used_when_version_list_has_no_allowed_release() {
            assert_eq!(
                latest_version_from_response(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.1","yanked":false},{"num":"1.0.229","yanked":true}]}"#,
                ),
                Some("1.0.228".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.2","yanked":true}]}"#,
                    true,
                ),
                Some("2.0.0-alpha.1".to_owned())
            );
        }

        #[test]
        fn cargo_crate_metadata_uses_max_version_when_prereleases_are_visible() {
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"}}"#,
                    true,
                ),
                Some("2.0.0-alpha.1".to_owned())
            );
        }

        #[test]
        fn deno_latest_field_falls_back_when_yanked_or_prerelease() {
            assert_eq!(
                latest_version_from_response(
                    Deno,
                    "@std/assert",
                    r#"{"latest":"1.3.0","versions":{"1.2.0":{},"1.3.0":{"yanked":true}}}"#,
                ),
                Some("1.2.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Deno,
                    "@std/assert",
                    r#"{"latest":"1.3.0-rc.1","versions":{"1.2.0":{},"1.3.0-rc.1":{}}}"#,
                ),
                Some("1.2.0".to_owned())
            );
        }

        #[test]
        fn reads_latest_versions_from_dub_registry_responses() {
            assert_latest(
                Dub,
                "vibe-d",
                r#"{"versions":[{"version":"~master"},{"version":"1.0.0"},{"version":"1.1.0-beta.1"},{"version":"1.0.2"}]}"#,
                "1.0.2",
            );
            assert_eq!(
        latest_version_for_requirement(
            Dub,
            "vibe-d",
            "~master",
            r#"{"versions":[{"version":"~master"},{"version":"0.9.0"},{"version":"0.8.0"}]}"#,
        )
        .as_deref(),
        Some("~master")
    );
            assert_latest(
                Dub,
                "vibe-d",
                r#"["~master","1.0.0","1.1.0-beta.1","1.0.2"]"#,
                "1.0.2",
            );
            assert_eq!(
                latest_version_for_requirement(
                    Dub,
                    "vibe-d",
                    "~master",
                    r#"["~master","0.9.0","0.8.0"]"#,
                )
                .as_deref(),
                Some("~master")
            );
            assert_latest(
                Dub,
                "vibe-d",
                r#"{"versions":[{"version":"~main"}]}"#,
                "~main",
            );
        }

        #[test]
        fn reads_latest_versions_from_text_registry_responses() {
            assert_latest(Go, "go.uber.org/zap", "v1.0.0\nv1.2.0\n", "v1.2.0");
            assert_latest(
                Go,
                "go.uber.org/zap",
                "\nv0.32.3\nv0.19.10\n\nv0.26.0\n",
                "v0.32.3",
            );
            assert_latest(
                Go,
                "github.com/docker/cli",
                "v26.1.3+incompatible\nv27.0.0+incompatible\n",
                "v27.0.0",
            );
            assert_latest(
                Maven,
                "org.springframework:spring-core",
                r"<metadata><versioning><versions><version>5.0.0</version><version>6.0.0-M1</version><version>5.3.1</version></versions></versioning></metadata>",
                "5.3.1",
            );
            assert_latest(
                Helm,
                "apache",
                "apiVersion: v1\nentries:\n  apache:\n    - version: 11.4.1\n    - version: 12.0.0-beta.1\n    - version: 11.3.0\n  mysql:\n    - version: 9.0.0\n",
                "11.4.1",
            );
            assert_eq!(
                latest_version_from_response(
                    Maven,
                    "org.springframework:spring-core",
                    r"<metadata><versioning><release>6.1.1</release><latest>6.2.0-M1</latest></versioning></metadata>",
                ),
                None
            );
            assert_eq!(
                latest_version_from_response(
                    Maven,
                    "org.springframework:spring-core",
                    r"<metadata><versioning><latest>6.1.2</latest></versioning></metadata>",
                ),
                None
            );
            assert_latest(
                Python,
                "flask",
                r"<rss><channel><item><title>Flask 2.3.3</title></item><item><title>Flask 3.0.0</title></item></channel></rss>",
                "3.0.0",
            );
            assert_latest(
                Python,
                "pip",
                r"<rss><channel><item><title>25.0.1</title></item><item><title>24.3.1</title></item></channel></rss>",
                "25.0.1",
            );
            assert_latest(
                Haxelib,
                "tink_core",
                r"<main><code>haxelib install tink_core 2.0.0</code><code>haxelib install tink_core 2.1.0-rc.1</code><code>haxelib install tink_core 1.9.0</code></main>",
                "2.0.0",
            );
        }

        #[test]
        fn python_rss_two_segment_latest_is_coerced_to_three_segments() {
            assert_eq!(
                latest_version_from_response(
                    Python,
                    "pip",
                    r"<rss><channel><item><title>25.0</title></item><item><title>24.3</title></item></channel></rss>",
                ),
                Some("25.0.0".to_owned())
            );
        }

        #[test]
        fn ignores_invalid_python_rss_responses() {
            assert_eq!(latest_version_from_response(Python, "pip", "<rss>"), None);
        }

        #[test]
        fn falls_back_when_registry_latest_tag_is_prerelease() {
            assert_eq!(
                latest_version_from_response(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"2.0.0-alpha.1"},"versions":[{"num":"1.0.228","yanked":false},{"num":"2.0.0-alpha.1","yanked":false}]}"#,
                ),
                Some("1.0.228".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Python,
                    "flask",
                    r#"{"info":{"version":"4.0.0rc1"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#,
                ),
                Some("3.0.0".to_owned())
            );
            assert_eq!(
                latest_version_from_response(
                    Pub,
                    "http",
                    r#"{"latest":{"version":"2.0.0-dev.1"}}"#,
                ),
                None
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Pub,
                    "http",
                    r#"{"latest":{"version":"2.0.0-dev.1"}}"#,
                    true,
                ),
                Some("2.0.0-dev.1".to_owned())
            );
        }

        #[test]
        fn npm_latest_dist_tag_uses_prerelease_tagged_version() {
            assert_eq!(
                latest_version_from_response(
                    Npm,
                    "typescript",
                    r#"{"dist-tags":{"latest":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
                ),
                Some("7.0.0-beta.1".to_owned())
            );
        }

        #[test]
        fn can_pick_prerelease_versions() {
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Cargo,
                    "serde",
                    r#"{"crate":{"max_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.1","yanked":true},{"num":"1.1.0-beta.1","yanked":false},{"num":"1.0.228","yanked":false}]}"#,
                    true,
                ),
                Some("1.1.0-beta.1".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Dotnet,
                    "Newtonsoft.Json",
                    r#"{"versions":["13.0.1","14.0.0-beta.1","13.0.3"]}"#,
                    true,
                ),
                Some("14.0.0-beta.1".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Npm,
                    "typescript",
                    r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
                    true,
                ),
                Some("6.0.3".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Python,
                    "flask",
                    r#"{"info":{"version":"3.0.0"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#,
                    true,
                ),
                Some("4.0.0rc1".to_owned())
            );
            assert_eq!(
                latest_version_from_response_with_prereleases(
                    Ruby,
                    "rails",
                    r#"[{"number":"8.0.4"},{"number":"8.1.0.beta1"}]"#,
                    true,
                ),
                Some("8.1.0.beta1".to_owned())
            );
        }

        #[test]
        fn filters_prerelease_tags_from_registry_responses() {
            assert_eq!(
                latest_version_with_tags(
                    Npm,
                    "typescript",
                    r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{},"8.0.0-rc.1":{}}}"#,
                    &["beta".to_owned()],
                ),
                Some("6.0.3".to_owned())
            );
            assert_eq!(
                latest_version_with_tags(
                    Deno,
                    "@std/assert",
                    r#"{"versions":{"1.0.0":{},"1.1.0-alpha.1":{},"1.1.0-beta.1":{}}}"#,
                    &["alpha".to_owned()],
                ),
                Some("1.1.0-alpha.1".to_owned())
            );
            assert_eq!(
                latest_version_with_tags(
                    Docker,
                    "node",
                    r#"{"results":[{"name":"24.1.0","tag_status":"active","digest":"sha256-24"},{"name":"25.0.0-beta.1","tag_status":"active","digest":"sha256-beta"},{"name":"25.0.0-rc.1","tag_status":"active","digest":"sha256-rc"}]}"#,
                    &["beta".to_owned()],
                ),
                Some("25.0.0-beta.1".to_owned())
            );
        }
    }
    mod vulnerability {
        use super::{
            vulnerability_advisories_from_response, vulnerability_request_body,
            vulnerability_titles_from_response, vulnerability_url,
            vulnerability_version_from_requirement,
        };
        use versionlens_model::Ecosystem::*;

        #[test]
        fn reads_osv_vulnerabilities_for_exact_versions() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "left-pad",
                "1.0.0",
                &super::osv_range_response("OSV-1", "left-pad prototype issue", "0", "1.1.0"),
            );

            assert_eq!(titles, vec!["OSV-1: left-pad prototype issue"]);
        }

        #[test]
        fn reports_returned_osv_vulnerabilities_for_other_ecosystems_like_upstream() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "requests",
                "2.32.0",
                r#"{
          "vulns": [{
            "id": "OSV-PYPI",
            "summary": "python requests issue",
            "affected": [{
              "package": { "ecosystem": "PyPI", "name": "requests" },
              "versions": ["2.32.0"]
            }]
          }, {
            "id": "OSV-NPM",
            "summary": "npm requests issue",
            "affected": [{
              "package": { "ecosystem": "npm", "name": "requests" },
              "versions": ["2.32.0"]
            }]
          }]
        }"#,
            );

            assert_eq!(
                titles,
                vec![
                    "OSV-PYPI: python requests issue",
                    "OSV-NPM: npm requests issue",
                ]
            );
        }

        #[test]
        fn uses_osv_vulnerability_url_for_advisories() {
            let advisories = vulnerability_advisories_from_response(
                Npm,
                "left-pad",
                "1.0.0",
                r#"{
          "vulns": [{
            "id": "OSV-URL",
            "summary": "left-pad advisory issue",
            "references": [
              { "type": "WEB", "url": "https://example.com/advisory" }
            ],
            "affected": [{
              "package": { "name": "left-pad" },
              "versions": ["1.0.0"]
            }]
          }]
        }"#,
            );

            assert_eq!(advisories.len(), 1);
            assert_eq!(advisories[0].id, "OSV-URL");
            assert_eq!(advisories[0].title, "OSV-URL: left-pad advisory issue");
            assert_eq!(
                advisories[0].url.as_deref(),
                Some("https://osv.dev/vulnerability/OSV-URL")
            );
        }

        #[test]
        fn builds_osv_advisory_url_from_id_when_references_are_missing() {
            let advisories = vulnerability_advisories_from_response(
                Npm,
                "left-pad",
                "1.0.0",
                r#"{
          "vulns": [{
            "id": "OSV-FALLBACK",
            "summary": "left-pad fallback issue",
            "affected": [{
              "package": { "name": "left-pad" },
              "versions": ["1.0.0"]
            }]
          }]
        }"#,
            );

            assert_eq!(
                advisories[0].url.as_deref(),
                Some("https://osv.dev/vulnerability/OSV-FALLBACK")
            );
        }

        #[test]
        fn reports_withdrawn_osv_vulnerabilities_like_upstream() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "left-pad",
                "1.0.0",
                r#"{
          "vulns": [{
            "id": "OSV-WITHDRAWN",
            "summary": "withdrawn left-pad issue",
            "withdrawn": "2026-01-01T00:00:00Z",
            "affected": [{
              "package": { "name": "left-pad" },
              "ranges": [{
                "events": [
                  { "introduced": "0" }
                ]
              }]
            }]
          }, {
            "id": "OSV-ACTIVE",
            "summary": "active left-pad issue",
            "affected": [{
              "package": { "name": "left-pad" },
              "ranges": [{
                "events": [
                  { "introduced": "0" }
                ]
              }]
            }]
          }]
        }"#,
            );

            assert_eq!(
                titles,
                vec![
                    "OSV-WITHDRAWN: withdrawn left-pad issue",
                    "OSV-ACTIVE: active left-pad issue",
                ]
            );
        }

        #[test]
        fn reports_osv_vulnerabilities_without_matching_affected_metadata_like_upstream() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "left-pad",
                "1.0.0",
                r#"{
          "vulns": [{
            "id": "OSV-NO-MATCHING-AFFECTED",
            "summary": "returned OSV issue",
            "affected": [{
              "package": { "ecosystem": "PyPI", "name": "requests" },
              "versions": ["2.32.0"]
            }]
          }]
        }"#,
            );

            assert_eq!(titles, vec!["OSV-NO-MATCHING-AFFECTED: returned OSV issue"]);
        }

        #[test]
        fn defaults_osv_vulnerability_title_when_summary_is_missing() {
            for summary in ["", r#""summary": "", "#] {
                let titles = vulnerability_titles_from_response(
                    Npm,
                    "left-pad",
                    "1.0.0",
                    &format!(
                        r#"{{
                  "vulns": [{{
                    "id": "GHSA-2",
                    {summary}
                    "affected": [{{
                      "package": {{ "name": "left-pad" }},
                      "ranges": [{{
                        "events": [
                          {{ "introduced": "0" }},
                          {{ "fixed": "1.1.0" }}
                        ]
                      }}]
                    }}]
                  }}]
                }}"#
                    ),
                );

                assert_eq!(titles, vec!["GHSA-2: No summary available"]);
            }
        }

        #[test]
        fn reads_osv_vulnerabilities_for_range_requirements() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "left-pad",
                "^1.0.0",
                &super::osv_range_response("OSV-1", "left-pad prototype issue", "0", "1.1.0"),
            );

            assert_eq!(titles, vec!["OSV-1: left-pad prototype issue"]);
        }

        #[test]
        fn reads_osv_vulnerabilities_for_open_ended_ranges() {
            let body = r#"{
      "vulns": [{
        "id": "OSV-OPEN",
        "summary": "left-pad open issue",
        "affected": [{
          "package": { "name": "left-pad" },
          "ranges": [{
            "events": [
              { "introduced": "1.0.0" }
            ]
          }]
        }]
      }]
    }"#;

            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "1.2.0", body),
                vec!["OSV-OPEN: left-pad open issue"]
            );
            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "0.9.0", body),
                vec!["OSV-OPEN: left-pad open issue"]
            );
        }

        #[test]
        fn reads_osv_vulnerabilities_for_last_affected_ranges() {
            let body = r#"{
      "vulns": [{
        "id": "OSV-LAST",
        "summary": "left-pad bounded issue",
        "affected": [{
          "package": { "name": "left-pad" },
          "ranges": [{
            "events": [
              { "introduced": "1.0.0" },
              { "last_affected": "1.2.0" }
            ]
          }]
        }]
      }]
    }"#;

            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "1.2.0", body),
                vec!["OSV-LAST: left-pad bounded issue"]
            );
            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "1.2.1", body),
                vec!["OSV-LAST: left-pad bounded issue"]
            );
        }

        #[test]
        fn reads_osv_vulnerabilities_for_limit_ranges() {
            let body = r#"{
      "vulns": [{
        "id": "OSV-LIMIT",
        "summary": "left-pad limited issue",
        "affected": [{
          "package": { "name": "left-pad" },
          "ranges": [{
            "events": [
              { "introduced": "1.0.0" },
              { "limit": "2.0.0" }
            ]
          }]
        }]
      }]
    }"#;

            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "1.9.0", body),
                vec!["OSV-LIMIT: left-pad limited issue"]
            );
            assert_eq!(
                vulnerability_titles_from_response(Npm, "left-pad", "2.0.0", body),
                vec!["OSV-LIMIT: left-pad limited issue"]
            );
        }

        #[test]
        fn reads_osv_vulnerabilities_from_affected_versions() {
            let titles = vulnerability_titles_from_response(
                Npm,
                "left-pad",
                "^1.0.0",
                r#"{
          "vulns": [{
            "id": "OSV-2",
            "summary": "left-pad listed version issue",
            "affected": [{
              "package": { "name": "left-pad" },
              "versions": ["0.9.0", "1.0.0"]
            }]
          }]
        }"#,
            );

            assert_eq!(titles, vec!["OSV-2: left-pad listed version issue"]);
        }

        #[test]
        #[expect(
            clippy::too_many_lines,
            reason = "table-driven manifest coverage stays readable as one scenario"
        )]
        fn builds_osv_query_for_supported_requirements() {
            assert_eq!(vulnerability_url(Npm), Some("https://api.osv.dev/v1/query"));
            assert_eq!(
                vulnerability_request_body(Cargo, "serde", "1.0.0"),
                Some(
                    r#"{"package":{"ecosystem":"crates.io","name":"serde"},"version":"1.0.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
        vulnerability_request_body(Composer, "phpunit/phpunit", "1.0.0"),
        Some(
            r#"{"package":{"ecosystem":"Packagist","name":"phpunit/phpunit"},"version":"1.0.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(Dotnet, "Newtonsoft.Json", "1.0.0"),
        Some(
            r#"{"package":{"ecosystem":"NuGet","name":"Newtonsoft.Json"},"version":"1.0.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(Go, "golang.org/x/term", "1.0.0"),
        Some(
            r#"{"package":{"ecosystem":"Go","name":"golang.org/x/term"},"version":"1.0.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(Go, "golang.org/x/term", "v1.0.0"),
        Some(
            r#"{"package":{"ecosystem":"Go","name":"golang.org/x/term"},"version":"v1.0.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
                vulnerability_request_body(Hex, "plug", "~> 1.20.0"),
                Some(
                    r#"{"package":{"ecosystem":"Hex","name":"plug"},"version":"1.20.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Hackage, "process", ">= 1.6.22.0"),
                Some(
                    r#"{"package":{"ecosystem":"Hackage","name":"process"},"version":"1.6.22.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Julia, "Example", "1.2.3"),
                Some(
                    r#"{"package":{"ecosystem":"Julia","name":"Example"},"version":"1.2.3"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Cran, "dplyr", "1.1.4"),
                Some(
                    r#"{"package":{"ecosystem":"CRAN","name":"dplyr"},"version":"1.1.4"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Cpan, "Plack", "1.0054"),
                Some(
                    r#"{"package":{"ecosystem":"CPAN","name":"Plack"},"version":"1.0054"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Opam, "lwt", "5.9.0"),
                Some(
                    r#"{"package":{"ecosystem":"opam","name":"lwt"},"version":"5.9.0"}"#.to_owned()
                )
            );
            assert_eq!(
        vulnerability_request_body(Swift, "apple/swift-nio", "2.65.0"),
        Some(
            r#"{"package":{"ecosystem":"SwiftURL","name":"github.com/apple/swift-nio"},"version":"2.65.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(Swift, "github.com/apple/swift-nio", "2.65.0"),
        Some(
            r#"{"package":{"ecosystem":"SwiftURL","name":"github.com/apple/swift-nio"},"version":"2.65.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(Maven, "org.test:example", "1.0.0"),
        Some(
            r#"{"package":{"ecosystem":"Maven","name":"org.test:example"},"version":"1.0.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
        vulnerability_request_body(
            Maven,
            "com.android.application:com.android.application.gradle.plugin",
            "8.12.0",
        ),
        Some(
            r#"{"package":{"ecosystem":"Maven","name":"com.android.application:com.android.application.gradle.plugin"},"version":"8.12.0"}"#
                .to_owned()
        )
    );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "1.0.0"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.0.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(vulnerability_url(Deno), None);
            assert_eq!(
                vulnerability_request_body(Deno, "@std/assert", "1.0.0"),
                None
            );
            assert_eq!(
                vulnerability_request_body(Python, "requests", "== 2.32.0"),
                Some(
                    r#"{"package":{"ecosystem":"PyPI","name":"requests"},"version":"2.32.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Pub, "http", "1.0.0"),
                Some(
                    r#"{"package":{"ecosystem":"Pub","name":"http"},"version":"1.0.0"}"#.to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Ruby, "rails", "1.0.0"),
                Some(
                    r#"{"package":{"ecosystem":"RubyGems","name":"rails"},"version":"1.0.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(vulnerability_url(Docker), None);
            assert_eq!(vulnerability_request_body(Docker, "ubuntu", "24.04"), None);
            assert_eq!(vulnerability_url(Dub), None);
            assert_eq!(vulnerability_request_body(Dub, "vibe-d", "1.0.0"), None);
            assert_eq!(
                vulnerability_url(Conan),
                Some("https://api.osv.dev/v1/query")
            );
            assert_eq!(
                vulnerability_request_body(Conan, "zlib", "1.3.1"),
                Some(r#"{"package":{"purl":"pkg:conan/zlib"},"version":"1.3.1"}"#.to_owned())
            );
            assert_eq!(vulnerability_url(LuaRocks), None);
            assert_eq!(
                vulnerability_request_body(LuaRocks, "luasocket", "3.1.0-1"),
                None
            );
            assert_eq!(vulnerability_url(Haxelib), None);
            assert_eq!(
                vulnerability_request_body(Haxelib, "tink_core", "2.0.0"),
                None
            );
            assert_eq!(
                vulnerability_url(CocoaPods),
                Some("https://api.osv.dev/v1/query")
            );
            assert_eq!(
                vulnerability_request_body(CocoaPods, "AFNetworking", "4.0.1"),
                Some(
                    r#"{"package":{"purl":"pkg:cocoapods/AFNetworking"},"version":"4.0.1"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "^1.0.0"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.0.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "> 7.0.0 < 7.6.0"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"7.0.0"}"#
                        .to_owned()
                )
            );
            assert_eq!(
        vulnerability_request_body(Npm, "left-pad", "^1.2.3-beta-1+build-7"),
        Some(
            r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.2.3-beta-1+build-7"}"#
                .to_owned()
        )
    );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "~1.2.x"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.2"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "v1.2.3-beta.1"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.2.3-beta"}"#
                        .to_owned()
                )
            );
            assert_eq!(
                vulnerability_request_body(Npm, "left-pad", "1.2.3-rc.1+build.7"),
                Some(
                    r#"{"package":{"ecosystem":"npm","name":"left-pad"},"version":"1.2.3-rc"}"#
                        .to_owned()
                )
            );
            assert_eq!(vulnerability_request_body(Npm, "left-pad", "latest"), None);
        }

        #[test]
        fn extracts_osv_query_versions_for_diagnostics() {
            assert_eq!(
                vulnerability_version_from_requirement("1.2.3-rc.1+build.7").as_deref(),
                Some("1.2.3-rc")
            );
            assert_eq!(
                vulnerability_version_from_requirement("^1.2.3-beta.1+build.7").as_deref(),
                Some("1.2.3-beta")
            );
            assert_eq!(
                vulnerability_version_from_requirement("> 7.0.0 < 7.6.0").as_deref(),
                Some("7.0.0")
            );
            assert_eq!(vulnerability_version_from_requirement("latest"), None);
        }
    }
    mod vulnerability_identity {
        use super::{
            vulnerability_request_body, vulnerability_titles_from_response, vulnerability_url,
        };
        use versionlens_model::Ecosystem::*;

        #[test]
        fn filters_osv_vulnerabilities_by_npm_purl_identity() {
            let body = r#"{
      "versions": { "1.0.0": {} },
      "vulns": [{
        "id": "OSV-PURL-MATCH",
        "summary": "scoped package issue",
        "affected": [{
          "package": { "ecosystem": "npm", "purl": "pkg:npm/%40scope/pkg" },
          "versions": ["1.0.0"]
        }]
      }, {
        "id": "OSV-PURL-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "npm", "purl": "pkg:npm/other-pkg" },
          "versions": ["1.0.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Npm, "@scope/pkg", "1.0.0", body);

            assert_eq!(titles, vec!["OSV-PURL-MATCH: scoped package issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_pypi_purl_identity() {
            let body = r#"{
      "versions": { "2.32.0": {} },
      "vulns": [{
        "id": "OSV-PYPI-MATCH",
        "summary": "requests issue",
        "affected": [{
          "package": { "ecosystem": "PyPI", "purl": "pkg:pypi/requests" },
          "versions": ["2.32.0"]
        }]
      }, {
        "id": "OSV-PYPI-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "PyPI", "purl": "pkg:pypi/other" },
          "versions": ["2.32.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Python, "requests", "2.32.0", body);

            assert_eq!(titles, vec!["OSV-PYPI-MATCH: requests issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_rubygems_purl_identity() {
            let body = r#"{
      "versions": { "7.1.0": {} },
      "vulns": [{
        "id": "OSV-RUBY-MATCH",
        "summary": "rails issue",
        "affected": [{
          "package": { "ecosystem": "RubyGems", "purl": "pkg:gem/rails" },
          "versions": ["7.1.0"]
        }]
      }, {
        "id": "OSV-RUBY-MISMATCH",
        "summary": "other gem issue",
        "affected": [{
          "package": { "ecosystem": "RubyGems", "purl": "pkg:gem/rack" },
          "versions": ["7.1.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Ruby, "rails", "7.1.0", body);

            assert_eq!(titles, vec!["OSV-RUBY-MATCH: rails issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_go_purl_identity_with_v_prefixed_versions() {
            let body = r#"{
      "versions": { "v1.0.0": {} },
      "vulns": [{
        "id": "OSV-GO-MATCH",
        "summary": "go module issue",
        "affected": [{
          "package": { "ecosystem": "Go", "purl": "pkg:golang/golang.org/x/term" },
          "versions": ["v1.0.0"]
        }]
      }, {
        "id": "OSV-GO-MISMATCH",
        "summary": "other module issue",
        "affected": [{
          "package": { "ecosystem": "Go", "purl": "pkg:golang/golang.org/x/net" },
          "versions": ["v1.0.0"]
        }]
      }]
    }"#;

            let titles =
                vulnerability_titles_from_response(Go, "golang.org/x/term", "v1.0.0", body);

            assert_eq!(titles, vec!["OSV-GO-MATCH: go module issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_maven_purl_identity() {
            let body = r#"{
      "versions": { "6.1.0": {} },
      "vulns": [{
        "id": "OSV-MAVEN-MATCH",
        "summary": "spring core issue",
        "affected": [{
          "package": { "ecosystem": "Maven", "purl": "pkg:maven/org.springframework/spring-core" },
          "versions": ["6.1.0"]
        }]
      }, {
        "id": "OSV-MAVEN-MISMATCH",
        "summary": "spring web issue",
        "affected": [{
          "package": { "ecosystem": "Maven", "purl": "pkg:maven/org.springframework/spring-web" },
          "versions": ["6.1.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(
                Maven,
                "org.springframework:spring-core",
                "6.1.0",
                body,
            );

            assert_eq!(titles, vec!["OSV-MAVEN-MATCH: spring core issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_gradle_plugin_marker_maven_purl_identity() {
            let body = r#"{
      "versions": { "8.12.0": {} },
      "vulns": [{
        "id": "OSV-GRADLE-PLUGIN-MATCH",
        "summary": "android plugin issue",
        "affected": [{
          "package": {
            "ecosystem": "Maven",
            "purl": "pkg:maven/com.android.application/com.android.application.gradle.plugin"
          },
          "versions": ["8.12.0"]
        }]
      }, {
        "id": "OSV-GRADLE-PLUGIN-MISMATCH",
        "summary": "other plugin issue",
        "affected": [{
          "package": {
            "ecosystem": "Maven",
            "purl": "pkg:maven/com.android.library/com.android.library.gradle.plugin"
          },
          "versions": ["8.12.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(
                Maven,
                "com.android.application:com.android.application.gradle.plugin",
                "8.12.0",
                body,
            );

            assert_eq!(
                titles,
                vec!["OSV-GRADLE-PLUGIN-MATCH: android plugin issue"]
            );
        }
        #[test]
        fn filters_osv_vulnerabilities_by_hex_purl_identity() {
            let body = r#"{
      "versions": { "1.20.0": {} },
      "vulns": [{
        "id": "OSV-HEX-MATCH",
        "summary": "plug issue",
        "affected": [{
          "package": { "ecosystem": "Hex", "purl": "pkg:hex/plug" },
          "versions": ["1.20.0"]
        }]
      }, {
        "id": "OSV-HEX-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "Hex", "purl": "pkg:hex/cowboy" },
          "versions": ["1.20.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Hex, "plug", "1.20.0", body);

            assert_eq!(titles, vec!["OSV-HEX-MATCH: plug issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_cpan_purl_identity() {
            let body = r#"{
      "versions": { "1.0054": {} },
      "vulns": [{
        "id": "OSV-CPAN-MATCH",
        "summary": "Plack issue",
        "affected": [{
          "package": { "ecosystem": "CPAN", "purl": "pkg:cpan/Plack" },
          "versions": ["1.0054"]
        }]
      }, {
        "id": "OSV-CPAN-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "CPAN", "purl": "pkg:cpan/Other" },
          "versions": ["1.0054"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Cpan, "Plack", "1.0054", body);

            assert_eq!(titles, vec!["OSV-CPAN-MATCH: Plack issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_opam_purl_identity() {
            let body = r#"{
      "versions": { "5.9.0": {} },
      "vulns": [{
        "id": "OSV-OPAM-MATCH",
        "summary": "lwt issue",
        "affected": [{
          "package": { "ecosystem": "opam", "purl": "pkg:opam/lwt" },
          "versions": ["5.9.0"]
        }]
      }, {
        "id": "OSV-OPAM-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "opam", "purl": "pkg:opam/cohttp" },
          "versions": ["5.9.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(Opam, "lwt", "5.9.0", body);

            assert_eq!(titles, vec!["OSV-OPAM-MATCH: lwt issue"]);
        }
        #[test]
        fn filters_osv_vulnerabilities_by_swift_purl_identity() {
            let body = r#"{
      "versions": { "2.65.0": {} },
      "vulns": [{
        "id": "OSV-SWIFT-MATCH",
        "summary": "swift-nio issue",
        "affected": [{
          "package": { "ecosystem": "SwiftURL", "purl": "pkg:swift/github.com/apple/swift-nio" },
          "versions": ["2.65.0"]
        }]
      }, {
        "id": "OSV-SWIFT-MISMATCH",
        "summary": "other package issue",
        "affected": [{
          "package": { "ecosystem": "SwiftURL", "purl": "pkg:swift/github.com/apple/swift-log" },
          "versions": ["2.65.0"]
        }]
      }]
    }"#;

            let titles = vulnerability_titles_from_response(
                Swift,
                "github.com/apple/swift-nio",
                "2.65.0",
                body,
            );

            assert_eq!(titles, vec!["OSV-SWIFT-MATCH: swift-nio issue"]);
        }

        #[test]
        fn filters_osv_vulnerabilities_by_cpp_git_repository_identity() {
            assert_eq!(vulnerability_url(Cpp), Some("https://api.osv.dev/v1/query"));
            assert_eq!(
        vulnerability_request_body(Cpp, "fmtlib/fmt", "11.1.4"),
        Some(
            r#"{"package":{"ecosystem":"GIT","name":"https://github.com/fmtlib/fmt.git"},"version":"11.1.4"}"#
                .to_owned()
        )
    );

            let body = r#"{
      "versions": { "11.1.4": {} },
      "vulns": [
        {
          "id": "OSV-CPP-MATCH",
          "summary": "fmt issue",
          "affected": [{
            "package": { "ecosystem": "GIT", "name": "fmt" },
            "ranges": [{
              "type": "GIT",
              "repo": "https://github.com/fmtlib/fmt.git",
              "events": [{ "introduced": "0" }]
            }],
            "versions": ["11.1.4"]
          }]
        },
        {
          "id": "OSV-CPP-MISMATCH",
          "summary": "other issue",
          "affected": [{
            "package": { "ecosystem": "GIT", "name": "other" },
            "ranges": [{
              "type": "GIT",
              "repo": "https://github.com/other/project.git",
              "events": [{ "introduced": "0" }]
            }],
            "versions": ["11.1.4"]
          }]
        }
      ]
    }"#;

            let titles = vulnerability_titles_from_response(
                Cpp,
                "https://github.com/fmtlib/fmt.git",
                "11.1.4",
                body,
            );

            assert_eq!(titles, vec!["OSV-CPP-MATCH: fmt issue"]);
        }

        #[test]
        fn builds_purl_osv_requests_for_supported_c_cpp_package_managers() {
            assert_eq!(
                vulnerability_request_body(Conan, "zlib", "1.3.1"),
                Some(r#"{"package":{"purl":"pkg:conan/zlib"},"version":"1.3.1"}"#.to_owned())
            );
            assert_eq!(
                vulnerability_request_body(Vcpkg, "fmt", "11.1.4"),
                Some(r#"{"package":{"purl":"pkg:vcpkg/fmt"},"version":"11.1.4"}"#.to_owned())
            );
            assert_eq!(
                vulnerability_request_body(CocoaPods, "AFNetworking", "4.0.1"),
                Some(
                    r#"{"package":{"purl":"pkg:cocoapods/AFNetworking"},"version":"4.0.1"}"#
                        .to_owned()
                )
            );
        }

        #[test]
        fn documents_osv_support_for_every_manifest_ecosystem() {
            let cases = [
                (Cargo, Some("crates.io")),
                (Composer, Some("Packagist")),
                (Deno, None),
                (Dotnet, Some("NuGet")),
                (Docker, None),
                (Dub, None),
                (Go, Some("Go")),
                (Maven, Some("Maven")),
                (Npm, Some("npm")),
                (Python, Some("PyPI")),
                (Pub, Some("Pub")),
                (Ruby, Some("RubyGems")),
                (Hex, Some("Hex")),
                (Opam, Some("opam")),
                (Hackage, Some("Hackage")),
                (Julia, Some("Julia")),
                (Cran, Some("CRAN")),
                (Conan, Some("purl")),
                (Vcpkg, Some("purl")),
                (Swift, Some("SwiftURL")),
                (Zig, None),
                (Nim, None),
                (LuaRocks, None),
                (Cpan, Some("CPAN")),
                (Haxelib, None),
                (Terraform, None),
                (Helm, None),
                (AnsibleGalaxy, None),
                (Bazel, None),
                (Nix, None),
                (Unity, None),
                (CocoaPods, Some("purl")),
                (Cpp, Some("GIT")),
            ];

            for (ecosystem, expected_osv_ecosystem) in cases {
                assert_eq!(
                    vulnerability_url(ecosystem).is_some(),
                    expected_osv_ecosystem.is_some()
                );

                let request = vulnerability_request_body(ecosystem, "example", "1.0.0");
                match expected_osv_ecosystem {
                    Some(expected) => {
                        let body = request.expect("supported OSV ecosystem should build query");
                        if expected == "purl" {
                            assert!(body.contains(r#""purl":"pkg:"#));
                        } else {
                            assert!(body.contains(&format!(r#""ecosystem":"{expected}""#)));
                        }
                    }
                    None => assert_eq!(request, None),
                }
            }
        }
    }
}
