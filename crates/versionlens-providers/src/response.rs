use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Composer, Cran, Docker, Dotnet, Hex, Maven, Npm, Python, Ruby};
mod cargo;
mod composer;
mod conan;
mod cpan;
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
mod zig;

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
use hex::hex_release_versions;
pub use npm::{npm_build_versions, npm_release_versions};
use python::python_release_versions;
use ruby::ruby_release_versions;
use xml::maven_release_versions;

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
        Python => python_release_versions(body),
        Ruby => ruby_release_versions(body),
        _ => vec![],
    }
}

#[cfg(test)]
mod tests;
