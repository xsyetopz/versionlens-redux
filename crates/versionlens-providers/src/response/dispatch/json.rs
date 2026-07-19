use serde_json::Value;
use serde_json::from_str;
use versionlens_model::Ecosystem;

use super::ResponseRequest;
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Composer, Conan, Cpan, Cpp, Deno, Docker, Dotnet, Dub,
    Hackage, Hex, Maven, Nim, Nix, Npm, Pub, Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};

mod ansible;
mod bazel;
mod cargo;
mod cocoapods;
mod composer;
mod conan;
mod cpan;
mod deno;
mod docker;
mod dotnet;
mod dub;
mod hackage;
mod hex;
mod maven;
mod nim;
mod nix;
mod npm;
mod pub_registry;
mod ruby;
mod swift;
mod terraform;
mod vcpkg;
mod zig;

use crate::response::cpp::latest_cpp_json_version;
use ansible::latest_ansible_json_response;
use bazel::latest_bazel_json_response;
use cargo::latest_cargo_json_response;
use cocoapods::latest_cocoapods_json_response;
use composer::latest_composer_json_response;
use conan::latest_conan_json_response;
use cpan::latest_cpan_json_response;
use deno::latest_deno_json_response;
use docker::latest_docker_json_response;
use dotnet::latest_dotnet_json_response;
use dub::latest_dub_json_response;
use hackage::latest_hackage_json_response;
use hex::latest_hex_json_response;
use maven::latest_maven_json_response;
use nim::latest_nim_json_response;
use nix::latest_nix_json_response;
use npm::latest_npm_json_response;
use pub_registry::latest_pub_json_response;
use ruby::latest_ruby_json_response;
use swift::latest_swift_json_response;
use terraform::latest_terraform_json_response;
use vcpkg::latest_vcpkg_json_response;
use zig::latest_zig_json_response;

pub(super) fn latest_json_response(
    ecosystem: Ecosystem,
    body: &str,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    let value = from_str::<Value>(body).ok()?;
    JSON_RESPONSE_PARSERS
        .iter()
        .find_map(|(known, parse)| (*known == ecosystem).then(|| parse(&value, request)))
        .flatten()
}

type JsonResponseParser = for<'a> fn(&Value, &ResponseRequest<'a>) -> Option<String>;

const JSON_RESPONSE_PARSERS: &[(Ecosystem, JsonResponseParser)] = &[
    (Cargo, latest_cargo_json_response),
    (AnsibleGalaxy, latest_ansible_json_response),
    (Bazel, latest_bazel_json_response),
    (Composer, latest_composer_json_response),
    (CocoaPods, latest_cocoapods_json_response),
    (Conan, latest_conan_json_response),
    (Cpan, latest_cpan_json_response),
    (Cpp, latest_cpp_json_response),
    (Deno, latest_deno_json_response),
    (Dotnet, latest_dotnet_json_response),
    (Docker, latest_docker_json_response),
    (Dub, latest_dub_json_response),
    (Hackage, latest_hackage_json_response),
    (Hex, latest_hex_json_response),
    (Maven, latest_maven_json_response),
    (Nim, latest_nim_json_response),
    (Nix, latest_nix_json_response),
    (Npm, latest_npm_json_response),
    (Unity, latest_npm_json_response),
    (Pub, latest_pub_json_response),
    (Ruby, latest_ruby_json_response),
    (Vcpkg, latest_vcpkg_json_response),
    (Swift, latest_swift_json_response),
    (Zig, latest_zig_json_response),
    (Terraform, latest_terraform_json_response),
];

fn latest_cpp_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    let tags = value.as_array()?;
    latest_cpp_json_version(
        &Value::Array(tags.to_owned()),
        request.include_prereleases,
        request.prerelease_tags,
    )
}
