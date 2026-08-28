use versionlens_model::ManifestKind;
use versionlens_model::ManifestKind::{
    DockerComposeYaml, DotnetXml, PnpmYaml, PythonPipfile, PythonPyprojectToml,
    PythonRequirementsTxt,
};

pub(super) fn classify_early_pattern_manifest(uri: &str) -> Option<ManifestKind> {
    if is_dotnet_xml_uri(uri) {
        return Some(DotnetXml);
    }
    if is_docker_compose_uri(uri) {
        return Some(DockerComposeYaml);
    }
    if is_pnpm_yaml_uri(uri) {
        return Some(PnpmYaml);
    }
    if let Some(kind) = classify_cpp_manifest(uri) {
        return Some(kind);
    }
    None
}

pub(super) fn classify_python_manifest(language_id: &str, uri: &str) -> Option<ManifestKind> {
    if language_id == "pip-requirements" || is_requirements_txt_uri(uri) {
        return Some(PythonRequirementsTxt);
    }
    if is_pipfile_uri(uri) {
        return Some(PythonPipfile);
    }
    if is_pyproject_toml_uri(uri) {
        return Some(PythonPyprojectToml);
    }
    None
}

use super::uri::{file_name, has_extension};
use versionlens_model::ManifestKind::{Cmake, MesonWrap};

pub(super) fn classify_cpp_manifest(uri: &str) -> Option<ManifestKind> {
    let name = file_name(uri)?;
    if has_extension(name, ["cmake"]) {
        return Some(Cmake);
    }
    if has_extension(name, ["wrap"]) {
        return Some(MesonWrap);
    }
    None
}

pub(in crate::classify) fn is_dockerfile_uri(uri: &str) -> bool {
    let Some(name) = file_name(uri) else {
        return false;
    };
    name.eq_ignore_ascii_case("Dockerfile") || ends_with_ignore_ascii_case(name, ".dockerfile")
}

pub(super) fn is_docker_compose_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if is_docker_compose_name(name))
}

fn is_docker_compose_name(name: &str) -> bool {
    [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yaml",
        "docker-compose.yml",
    ]
    .iter()
    .any(|item| name.eq_ignore_ascii_case(item))
        || is_docker_compose_variant_name(name)
        || is_dot_compose_yaml_name(name)
}

fn is_docker_compose_variant_name(name: &str) -> bool {
    (starts_with_ignore_ascii_case(name, "docker-compose.")
        || starts_with_ignore_ascii_case(name, "compose."))
        && has_extension(name, ["yaml", "yml"])
}

fn is_dot_compose_yaml_name(name: &str) -> bool {
    has_extension(name, ["yaml", "yml"])
        && name
            .rsplit_once('.')
            .is_some_and(|(stem, _)| ends_with_ignore_ascii_case(stem, ".compose"))
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|end| end.eq_ignore_ascii_case(suffix))
}

pub(super) fn is_dotnet_xml_uri(uri: &str) -> bool {
    if is_dotnet_generated_uri(uri) {
        return false;
    }

    matches!(
        file_name(uri),
        Some(name)
            if has_extension(name, ["csproj", "fsproj", "vbproj", "targets", "props"])
    )
}

fn is_dotnet_generated_uri(uri: &str) -> bool {
    uri.split('/')
        .any(|segment| segment.eq_ignore_ascii_case("obj") || segment.eq_ignore_ascii_case("bin"))
}

pub(super) fn is_requirements_txt_uri(uri: &str) -> bool {
    matches!(
        file_name(uri),
        Some(name) if is_python_requirements_or_constraints_name(name) && has_extension(name, ["txt"])
    )
}

pub(super) fn is_pipfile_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if name.eq_ignore_ascii_case("Pipfile"))
}

pub(super) fn is_pyproject_toml_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if name.eq_ignore_ascii_case("pyproject.toml"))
}

fn is_python_requirements_or_constraints_name(name: &str) -> bool {
    let name = name.to_lowercase();
    name.contains("requirements") || name.contains("constraints")
}

pub(super) fn is_pnpm_yaml_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if is_pnpm_yaml_name(name))
}

fn is_pnpm_yaml_name(name: &str) -> bool {
    [
        "pnpm-workspace.yaml",
        "pnpm-workspace.yml",
        ".yarnrc.yml",
        ".yarnrc.yaml",
    ]
    .iter()
    .any(|item| name.eq_ignore_ascii_case(item))
}
