use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{Composer, Cran, Docker, Pub, Ruby};

mod docker;
mod nuget;

use docker::is_supported_docker_dependency;
use nuget::contains_four_segment_nuget_version;

pub fn is_unsupported_dotnet_requirement(requirement: &str) -> bool {
    contains_four_segment_nuget_version(requirement.trim())
}

pub fn is_registry_requirement(ecosystem: Ecosystem, requirement: &str) -> bool {
    let requirement = requirement.trim();
    if ecosystem == Docker && requirement.starts_with("sha256:") {
        return false;
    }
    if ecosystem == Ruby && requirement.contains('/') {
        return false;
    }
    if ecosystem == Pub && requirement.contains('/') {
        return false;
    }

    !(requirement.contains('$')
        || requirement.starts_with('/')
        || requirement.starts_with("./")
        || requirement.starts_with("../")
        || requirement.starts_with("~/")
        || requirement.starts_with("file:")
        || requirement.starts_with("github:")
        || requirement.starts_with("gitlab:")
        || requirement.starts_with("bitbucket:")
        || requirement.starts_with("git+")
        || requirement.starts_with("git@")
        || requirement.starts_with("link:")
        || requirement.starts_with("portal:")
        || requirement.starts_with("exec:")
        || requirement.starts_with("patch:")
        || requirement.starts_with("path:")
        || requirement.starts_with("sdk:")
        || requirement.starts_with("workspace:")
        || requirement.starts_with("catalog:")
        || requirement.contains("://"))
}

pub fn is_registry_dependency(ecosystem: Ecosystem, name: &str, requirement: &str) -> bool {
    is_registry_requirement(ecosystem, requirement)
        && (ecosystem != Cran || name != "R")
        && (ecosystem != Docker || is_supported_docker_dependency(name))
        && (ecosystem != Composer || !is_composer_platform_dependency(name))
}

pub fn is_composer_platform_dependency(name: &str) -> bool {
    let name = name.trim();
    matches!(
        name,
        "php" | "composer" | "composer-plugin-api" | "composer-runtime-api"
    ) || name.starts_with("ext-")
        || name.starts_with("lib-")
}
