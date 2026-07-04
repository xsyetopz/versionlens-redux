use versionlens_parsers::Ecosystem;

use crate::{
    vulnerability_advisories_from_response, vulnerability_request_body,
    vulnerability_titles_from_response, vulnerability_url, vulnerability_version_from_requirement,
};

use super::{
    LatestVersionRequest, RegistryErrorStatus, build_versions_from_response, docker_tag_exists,
    http_status_message_from_code, latest_version_from_response,
    latest_version_from_response_for_request, latest_version_from_response_with_prereleases,
    npm_build_versions, npm_error_status_from_response, release_versions_from_response,
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

mod composer;
mod docker;
mod dotnet;
mod latest;
mod npm_errors;
mod vulnerability;
