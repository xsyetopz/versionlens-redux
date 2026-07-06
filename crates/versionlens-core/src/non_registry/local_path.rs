mod normalize;
mod requirement;

use normalize::resolve_local_path;
use requirement::local_requirement_path;
use versionlens_parsers::Dependency;
use versionlens_parsers::Ecosystem::{Docker, Pub, Ruby};

pub(super) struct LocalDependencyPath {
    pub(super) display: String,
    pub(super) resolved: String,
}

pub(super) fn local_dependency_path(
    dependency: &Dependency,
    document_uri: Option<&str>,
) -> Option<LocalDependencyPath> {
    let path = hosted_path_requirement(dependency)
        .or_else(|| local_requirement_path(&dependency.requirement))
        .or_else(|| ruby_bare_path_requirement(dependency))
        .or_else(|| pub_workspace_path_requirement(dependency))
        .or_else(|| docker_bare_build_path_requirement(dependency))?;
    let display = path.to_owned();
    let resolved = resolve_local_path(&display, document_uri);
    Some(LocalDependencyPath { display, resolved })
}

fn hosted_path_requirement(dependency: &Dependency) -> Option<&str> {
    let requirement = dependency.requirement.trim();
    (dependency.hosted_url.as_deref() == Some("path") && !requirement.is_empty())
        .then_some(requirement)
}

fn ruby_bare_path_requirement(dependency: &Dependency) -> Option<&str> {
    let requirement = dependency.requirement.trim();
    (dependency.ecosystem == Ruby && requirement.contains('/') && !requirement.contains("://"))
        .then_some(requirement)
}

fn pub_workspace_path_requirement(dependency: &Dependency) -> Option<&str> {
    let requirement = dependency.requirement.trim();
    (dependency.ecosystem == Pub
        && dependency.group == "workspace"
        && requirement.contains('/')
        && !requirement.contains("://"))
    .then_some(requirement)
}

fn docker_bare_build_path_requirement(dependency: &Dependency) -> Option<&str> {
    let requirement = dependency.requirement.trim();
    (dependency.ecosystem == Docker
        && dependency.group == "services.build"
        && !requirement.is_empty())
    .then_some(requirement)
}
