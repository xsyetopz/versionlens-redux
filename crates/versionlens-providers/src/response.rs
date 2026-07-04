mod cargo;
mod common;
mod composer;
mod deno;
mod dispatch;
mod docker;
mod dotnet;
mod dub;
mod errors;
mod github;
mod go;
mod npm;
mod pub_registry;
mod python;
mod ruby;
mod xml;

use composer::composer_release_versions;
pub use dispatch::{
    LatestVersionRequest, latest_version_from_response, latest_version_from_response_for_request,
    latest_version_from_response_with_prereleases,
};
use docker::docker_build_versions;
pub use docker::docker_tag_exists;
use dotnet::dotnet_release_versions;
pub use errors::{
    RegistryErrorStatus, http_status_message_from_code, npm_error_status_from_response,
};
pub use npm::{npm_build_versions, npm_release_versions};
use python::python_release_versions;
use ruby::ruby_release_versions;
use xml::maven_release_versions;

pub fn build_versions_from_response(
    ecosystem: versionlens_parsers::Ecosystem,
    body: &str,
    requirement: &str,
) -> Vec<String> {
    match ecosystem {
        versionlens_parsers::Ecosystem::Docker => docker_build_versions(body, requirement),
        versionlens_parsers::Ecosystem::Npm => npm_build_versions(body, requirement),
        _ => Vec::new(),
    }
}

pub fn release_versions_from_response(
    ecosystem: versionlens_parsers::Ecosystem,
    body: &str,
) -> Vec<String> {
    match ecosystem {
        versionlens_parsers::Ecosystem::Composer => composer_release_versions(body),
        versionlens_parsers::Ecosystem::Dotnet => dotnet_release_versions(body),
        versionlens_parsers::Ecosystem::Maven => maven_release_versions(body),
        versionlens_parsers::Ecosystem::Npm => npm_release_versions(body),
        versionlens_parsers::Ecosystem::Python => python_release_versions(body),
        versionlens_parsers::Ecosystem::Ruby => ruby_release_versions(body),
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests;
