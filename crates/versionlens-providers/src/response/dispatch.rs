use self::json::latest_json_response;
use self::text::latest_text_response;
use versionlens_model::Ecosystem;

mod json;
mod request;
mod text;

pub use request::LatestVersionRequest;
use request::ResponseRequest;

fn response_request_from_latest<'a>(request: &LatestVersionRequest<'a>) -> ResponseRequest<'a> {
    ResponseRequest {
        package: request.package,
        requirement: request.requirement,
        include_prereleases: request.include_prereleases,
        prerelease_tags: request.prerelease_tags,
    }
}

pub fn latest_version_from_response(
    ecosystem: Ecosystem,
    package: &str,
    body: &str,
) -> Option<String> {
    latest_version_from_response_with_prereleases(ecosystem, package, body, false)
}

pub fn latest_version_from_response_with_prereleases(
    ecosystem: Ecosystem,
    package: &str,
    body: &str,
    include_prereleases: bool,
) -> Option<String> {
    latest_version_from_response_for_request(LatestVersionRequest {
        ecosystem,
        package,
        requirement: "",
        body,
        include_prereleases,
        prerelease_tags: &[],
    })
}

pub fn latest_version_from_response_for_request(
    request: LatestVersionRequest<'_>,
) -> Option<String> {
    let parser_request = response_request_from_latest(&request);

    match request.ecosystem {
        Ecosystem::Cran
        | Ecosystem::Go
        | Ecosystem::Julia
        | Ecosystem::LuaRocks
        | Ecosystem::Opam
        | Ecosystem::Python
        | Ecosystem::Haxelib => {
            latest_text_response(request.ecosystem, request.body, &parser_request)
        }
        Ecosystem::Cpp => latest_json_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_text_response(request.ecosystem, request.body, &parser_request)),
        Ecosystem::Helm => latest_text_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_json_response(request.ecosystem, request.body, &parser_request)),
        Ecosystem::Maven => latest_json_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_text_response(request.ecosystem, request.body, &parser_request)),
        Ecosystem::Cargo
        | Ecosystem::AnsibleGalaxy
        | Ecosystem::Bazel
        | Ecosystem::Nix
        | Ecosystem::Composer
        | Ecosystem::CocoaPods
        | Ecosystem::Conan
        | Ecosystem::Cpan
        | Ecosystem::Deno
        | Ecosystem::Dotnet
        | Ecosystem::Docker
        | Ecosystem::Dub
        | Ecosystem::Hackage
        | Ecosystem::Hex
        | Ecosystem::Nim
        | Ecosystem::Npm
        | Ecosystem::Unity
        | Ecosystem::Pub
        | Ecosystem::Ruby
        | Ecosystem::Vcpkg
        | Ecosystem::Swift
        | Ecosystem::Zig
        | Ecosystem::Terraform
        | Ecosystem::GitHub => {
            latest_json_response(request.ecosystem, request.body, &parser_request)
        }
    }
}
