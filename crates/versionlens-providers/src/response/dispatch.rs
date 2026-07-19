use self::json::latest_json_response;
use self::text::latest_text_response;
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet,
    Dub, Go, Hackage, Haxelib, Helm, Hex, Julia, LuaRocks, Maven, Nim, Nix, Npm, Opam, Pub, Python,
    Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};

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
        Cran | Go | Julia | LuaRocks | Opam | Python | Haxelib => {
            latest_text_response(request.ecosystem, request.body, &parser_request)
        }
        Cpp => latest_json_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_text_response(request.ecosystem, request.body, &parser_request)),
        Helm => latest_text_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_json_response(request.ecosystem, request.body, &parser_request)),
        Maven => latest_json_response(request.ecosystem, request.body, &parser_request)
            .or_else(|| latest_text_response(request.ecosystem, request.body, &parser_request)),
        Cargo | AnsibleGalaxy | Bazel | Nix | Composer | CocoaPods | Conan | Cpan | Deno
        | Dotnet | Docker | Dub | Hackage | Hex | Nim | Npm | Unity | Pub | Ruby | Vcpkg
        | Swift | Zig | Terraform => {
            latest_json_response(request.ecosystem, request.body, &parser_request)
        }
    }
}
