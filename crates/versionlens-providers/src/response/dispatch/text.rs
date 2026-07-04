use versionlens_parsers::Ecosystem;

use super::ResponseRequest;
use crate::response::go::latest_go_version;
use crate::response::python::latest_python_version;
use crate::response::xml::latest_maven_version;

pub(super) fn latest_text_response(
    ecosystem: Ecosystem,
    body: &str,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    TEXT_RESPONSE_PARSERS
        .iter()
        .find_map(|(known, parse)| (*known == ecosystem).then(|| parse(body, request)))
        .flatten()
}

type TextResponseParser = for<'a> fn(&str, &ResponseRequest<'a>) -> Option<String>;

const TEXT_RESPONSE_PARSERS: &[(Ecosystem, TextResponseParser)] = &[
    (Ecosystem::Go, latest_go_text_response),
    (Ecosystem::Maven, latest_maven_text_response),
    (Ecosystem::Python, latest_python_text_response),
];

fn latest_go_text_response(body: &str, request: &ResponseRequest<'_>) -> Option<String> {
    latest_go_version(body, request.include_prereleases, request.prerelease_tags)
}

fn latest_maven_text_response(body: &str, request: &ResponseRequest<'_>) -> Option<String> {
    latest_maven_version(body, request.include_prereleases, request.prerelease_tags)
}

fn latest_python_text_response(body: &str, request: &ResponseRequest<'_>) -> Option<String> {
    latest_python_version(body, request.include_prereleases, request.prerelease_tags)
}
