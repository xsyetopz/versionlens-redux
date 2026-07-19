use serde_json::Value;
use serde_json::from_str;

use crate::{RegistryEndpoint, RegistryResponseKind};

use super::dispatch::{LatestVersionRequest, latest_version_from_response_for_request};
use super::go::{latest_go_module_latest_version, latest_go_module_list_version};
use super::python::{latest_python_version, python_release_versions};
use super::python_simple::{latest_python_simple_version, python_simple_release_versions};
use super::xml::{latest_python_rss_version, python_rss_release_versions};

pub fn latest_version_from_response_for_endpoint(
    endpoint: &RegistryEndpoint,
    request: LatestVersionRequest<'_>,
) -> Option<String> {
    match endpoint.response_kind {
        RegistryResponseKind::Ecosystem => latest_version_from_response_for_request(request),
        RegistryResponseKind::GoModuleList => latest_go_module_list_version(
            request.body,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        RegistryResponseKind::GoModuleLatest => latest_go_module_latest_version(request.body),
        RegistryResponseKind::PythonJson => python_json_payload(request.body).and_then(|_| {
            latest_python_version(
                request.body,
                request.include_prereleases,
                request.prerelease_tags,
            )
        }),
        RegistryResponseKind::PythonRss => latest_python_rss_version(
            request.body,
            request.include_prereleases,
            request.prerelease_tags,
        ),
        RegistryResponseKind::PythonSimple => latest_python_simple_version(
            request.body,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        ),
    }
}

pub fn release_versions_from_response_for_endpoint(
    endpoint: &RegistryEndpoint,
    ecosystem: versionlens_model::Ecosystem,
    package: &str,
    body: &str,
) -> Vec<String> {
    match endpoint.response_kind {
        RegistryResponseKind::PythonJson => python_json_payload(body)
            .map(|_| python_release_versions(body))
            .unwrap_or_default(),
        RegistryResponseKind::PythonRss => python_rss_release_versions(body),
        RegistryResponseKind::PythonSimple => python_simple_release_versions(body, package),
        RegistryResponseKind::GoModuleList | RegistryResponseKind::GoModuleLatest => vec![],
        RegistryResponseKind::Ecosystem => {
            super::release_versions_from_response_for_package(ecosystem, package, body)
        }
    }
}

fn python_json_payload(body: &str) -> Option<Value> {
    let value = from_str::<Value>(body).ok()?;
    (value.get("releases").is_some_and(Value::is_object)
        || value.pointer("/info/version").is_some_and(Value::is_string))
    .then_some(value)
}
