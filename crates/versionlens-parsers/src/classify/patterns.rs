use crate::model::ManifestKind;

mod docker;
mod dotnet;
mod python;
mod workspace;

pub(super) use docker::is_dockerfile_uri;

use docker::is_docker_compose_uri;
use dotnet::is_dotnet_xml_uri;
use python::{is_pipfile_uri, is_pyproject_toml_uri, is_requirements_txt_uri};
use workspace::is_pnpm_yaml_uri;

pub(super) fn classify_early_pattern_manifest(uri: &str) -> Option<ManifestKind> {
    if is_dotnet_xml_uri(uri) {
        return Some(ManifestKind::DotnetXml);
    }
    if is_docker_compose_uri(uri) {
        return Some(ManifestKind::DockerComposeYaml);
    }
    if is_pnpm_yaml_uri(uri) {
        return Some(ManifestKind::PnpmYaml);
    }
    None
}

pub(super) fn classify_python_manifest(language_id: &str, uri: &str) -> Option<ManifestKind> {
    if language_id == "pip-requirements" || is_requirements_txt_uri(uri) {
        return Some(ManifestKind::PythonRequirementsTxt);
    }
    if is_pipfile_uri(uri) {
        return Some(ManifestKind::PythonPipfile);
    }
    if is_pyproject_toml_uri(uri) {
        return Some(ManifestKind::PythonPyprojectToml);
    }
    None
}
