use versionlens_parsers::Ecosystem;

mod json;
mod request;
mod text;

pub use request::LatestVersionRequest;
use request::ResponseRequest;

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
    let parser_request = ResponseRequest::from_latest(&request);

    match request.ecosystem {
        Ecosystem::Go | Ecosystem::Python => {
            text::latest_text_response(request.ecosystem, request.body, &parser_request)
        }
        Ecosystem::Maven => {
            json::latest_json_response(request.ecosystem, request.body, &parser_request).or_else(
                || text::latest_text_response(request.ecosystem, request.body, &parser_request),
            )
        }
        Ecosystem::Cargo
        | Ecosystem::Composer
        | Ecosystem::Deno
        | Ecosystem::Dotnet
        | Ecosystem::Docker
        | Ecosystem::Dub
        | Ecosystem::Npm
        | Ecosystem::Pub
        | Ecosystem::Ruby => {
            json::latest_json_response(request.ecosystem, request.body, &parser_request)
        }
    }
}
