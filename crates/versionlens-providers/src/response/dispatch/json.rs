use serde_json::Value;
use versionlens_parsers::Ecosystem;

use super::ResponseRequest;

mod cargo;
mod composer;
mod deno;
mod docker;
mod dotnet;
mod dub;
mod maven;
mod npm;
mod pub_registry;
mod ruby;

use cargo::latest_cargo_json_response;
use composer::latest_composer_json_response;
use deno::latest_deno_json_response;
use docker::latest_docker_json_response;
use dotnet::latest_dotnet_json_response;
use dub::latest_dub_json_response;
use maven::latest_maven_json_response;
use npm::latest_npm_json_response;
use pub_registry::latest_pub_json_response;
use ruby::latest_ruby_json_response;

pub(super) fn latest_json_response(
    ecosystem: Ecosystem,
    body: &str,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    let value = serde_json::from_str::<Value>(body).ok()?;
    JSON_RESPONSE_PARSERS
        .iter()
        .find_map(|(known, parse)| (*known == ecosystem).then(|| parse(&value, request)))
        .flatten()
}

type JsonResponseParser = for<'a> fn(&Value, &ResponseRequest<'a>) -> Option<String>;

const JSON_RESPONSE_PARSERS: &[(Ecosystem, JsonResponseParser)] = &[
    (Ecosystem::Cargo, latest_cargo_json_response),
    (Ecosystem::Composer, latest_composer_json_response),
    (Ecosystem::Deno, latest_deno_json_response),
    (Ecosystem::Dotnet, latest_dotnet_json_response),
    (Ecosystem::Docker, latest_docker_json_response),
    (Ecosystem::Dub, latest_dub_json_response),
    (Ecosystem::Maven, latest_maven_json_response),
    (Ecosystem::Npm, latest_npm_json_response),
    (Ecosystem::Pub, latest_pub_json_response),
    (Ecosystem::Ruby, latest_ruby_json_response),
];
