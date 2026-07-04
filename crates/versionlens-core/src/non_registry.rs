use std::path::Path;

use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::{
    is_composer_platform_dependency, is_registry_dependency, is_unsupported_dotnet_requirement,
};
use versionlens_suggestions::{
    Suggestion, directory, directory_not_found, error, fixed, not_supported,
};

use crate::dependency::is_npm_package_manager;

mod fixed_spec;
mod git;
mod local_path;

use fixed_spec::is_fixed_spec;
use git::{is_git_dependency_requirement, is_unsupported_npm_git_requirement};
use local_path::{LocalDependencyPath, local_dependency_path};

pub(crate) fn known_non_registry_suggestion(
    dependency: Dependency,
    document_uri: Option<&str>,
) -> Result<Suggestion, Box<Dependency>> {
    if is_npm_bundle_name_only(&dependency) {
        return Ok(fixed(dependency, "bundled dependency".to_owned()));
    }
    if is_npm_package_manager(&dependency) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }
    if is_docker_build_path(&dependency) {
        if let Some(path) = local_dependency_path(&dependency, document_uri) {
            return Ok(local_directory_suggestion(dependency, path));
        }
    }
    if is_docker_argument_reference(&dependency) {
        return Ok(not_supported(dependency));
    }
    if is_registry_dependency(
        dependency.ecosystem,
        &dependency.name,
        &dependency.requirement,
    ) {
        return Err(Box::new(dependency));
    }
    if is_npm_bare_local_path(&dependency) {
        return Ok(error(dependency, "invalid version".to_owned()));
    }
    if is_unsupported_dotnet_version(&dependency) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }
    if let Some(path) = local_dependency_path(&dependency, document_uri) {
        return Ok(local_directory_suggestion(dependency, path));
    }
    if is_npm_unsupported_git_dependency(&dependency) {
        return Ok(not_supported(dependency));
    }
    if is_git_dependency_requirement(&dependency.requirement) {
        return Ok(fixed(dependency, "git repository".to_owned()));
    }
    if is_composer_platform(&dependency) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }
    if is_fixed_spec(&dependency.requirement) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }

    Err(Box::new(dependency))
}

pub(crate) fn deno_import_has_no_suggestions(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Deno
        && !dependency.requirement.starts_with("jsr:")
        && !dependency.requirement.starts_with("npm:")
}

fn local_directory_suggestion(dependency: Dependency, path: LocalDependencyPath) -> Suggestion {
    let LocalDependencyPath { display, resolved } = path;
    if Path::new(&resolved).exists() {
        return directory(dependency, display, resolved);
    }
    directory_not_found(dependency, display)
}

fn is_composer_platform(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Composer && is_composer_platform_dependency(&dependency.name)
}

fn is_npm_bundle_name_only(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Npm
        && dependency.requirement.is_empty()
        && matches!(
            dependency.group.as_str(),
            "bundledDependencies" | "bundleDependencies"
        )
}

fn is_npm_bare_local_path(dependency: &Dependency) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Ecosystem::Npm
        && (requirement.starts_with('/')
            || requirement.starts_with("./")
            || requirement.starts_with("../")
            || requirement.starts_with("~/"))
}

fn is_npm_unsupported_git_dependency(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Npm
        && is_unsupported_npm_git_requirement(&dependency.requirement)
}

fn is_docker_build_path(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Docker && dependency.group == "services.build"
}

fn is_docker_argument_reference(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Docker
        && (dependency.name.contains('$') || dependency.requirement.contains('$'))
}

fn is_unsupported_dotnet_version(dependency: &Dependency) -> bool {
    dependency.ecosystem == Ecosystem::Dotnet
        && is_unsupported_dotnet_requirement(&dependency.requirement)
}
