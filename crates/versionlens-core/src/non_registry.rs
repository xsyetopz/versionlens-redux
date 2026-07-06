use versionlens_parsers::Dependency;
use versionlens_parsers::Ecosystem::{
    AnsibleGalaxy, Bazel, CocoaPods, Composer, Cran, Deno, Docker, Dotnet, Go, Haxelib, Helm,
    LuaRocks, Maven, Nim, Nix, Npm, Swift, Terraform, Unity, Vcpkg, Zig,
};
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

#[expect(
    clippy::too_many_lines,
    reason = "central fixed-source dispatch keeps statuses ordered by ecosystem"
)]
pub(crate) fn known_non_registry_suggestion(
    dependency: Dependency,
    document_uri: Option<&str>,
) -> Result<Suggestion, Box<Dependency>> {
    if is_npm_name_only_metadata(&dependency) {
        let label = if dependency.group == "trustedDependencies" {
            "trusted dependency"
        } else {
            "bundled dependency"
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if is_paket_reference(&dependency) {
        return Ok(fixed(dependency, "paket reference".to_owned()));
    }
    if is_npm_package_manager(&dependency) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }
    if is_npm_override_reference(&dependency) {
        return Ok(fixed(dependency, "override reference".to_owned()));
    }
    if is_docker_build_path(&dependency) {
        if let Some(path) = local_dependency_path(&dependency, document_uri) {
            return Ok(local_directory_suggestion(dependency, path));
        }
    }
    if is_docker_argument_reference(&dependency) {
        return Ok(not_supported(dependency));
    }
    if is_npm_unsupported_protocol_dependency(&dependency) {
        return Ok(not_supported(dependency));
    }
    if is_composer_branch_alias_dependency(&dependency) {
        let requirement = format!(
            "{}{}",
            dependency.requirement, dependency.requirement_suffix
        );
        return Ok(fixed(dependency, requirement));
    }
    if dependency.hosted_url.as_deref() == Some("umbrella") {
        return Ok(fixed(dependency, "umbrella dependency".to_owned()));
    }
    if dependency.hosted_url.as_deref() == Some("stack-resolver") {
        return Ok(fixed(dependency, "stack resolver".to_owned()));
    }
    if dependency.ecosystem == Vcpkg && dependency.hosted_url.as_deref() == Some("baseline") {
        return Ok(fixed(dependency, "baseline dependency".to_owned()));
    }
    if dependency.ecosystem == Swift
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "git" | "path")
    {
        let label = if source == "path" {
            "local package"
        } else {
            "git repository"
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Zig
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "hash" | "path" | "url")
    {
        let label = match source {
            "path" => "local package",
            "hash" => "package hash",
            _ => "package URL",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Nim
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "head" | "git" | "path")
    {
        let label = match source {
            "path" => "local package",
            "git" => "git repository",
            _ => "moving branch",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == LuaRocks && dependency.name == "lua" {
        return Ok(fixed(dependency, "Lua runtime".to_owned()));
    }
    if dependency.ecosystem == Haxelib && dependency.hosted_url.as_deref() == Some("latest") {
        return Ok(fixed(dependency, "latest version".to_owned()));
    }
    if dependency.ecosystem == Terraform && dependency.hosted_url.as_deref() == Some("builtin") {
        return Ok(fixed(dependency, "built-in provider".to_owned()));
    }
    if dependency.ecosystem == Helm
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "file" | "repo-alias")
    {
        let label = if source == "file" {
            "local chart"
        } else {
            "repository alias"
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == AnsibleGalaxy && dependency.hosted_url.as_deref() == Some("git") {
        return Ok(fixed(dependency, "git repository".to_owned()));
    }
    if dependency.ecosystem == Bazel
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "git" | "archive" | "path")
    {
        let label = match source {
            "git" => "git repository",
            "archive" => "source archive",
            _ => "local module",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Unity
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "file" | "git" | "url")
    {
        let label = match source {
            "file" => "local package",
            "git" => "git repository",
            _ => "package URL",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == CocoaPods
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "latest" | "path" | "git" | "podspec")
    {
        let label = match source {
            "latest" => "latest version",
            "path" => "local pod",
            "git" => "git repository",
            _ => "podspec source",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Nix
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "git" | "path")
    {
        let label = if source == "path" {
            "local flake"
        } else {
            "git source"
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Maven && dependency.hosted_url.as_deref() == Some("version.ref") {
        return Ok(fixed(dependency, "version catalog reference".to_owned()));
    }
    if dependency.ecosystem == Maven && dependency.hosted_url.as_deref() == Some("version.alias") {
        return Ok(fixed(dependency, "version catalog alias".to_owned()));
    }
    if dependency.ecosystem == Maven
        && dependency.hosted_url.as_deref() == Some("scala-binary-version")
    {
        return Ok(fixed(dependency, "Scala binary version".to_owned()));
    }
    if dependency.ecosystem == Maven
        && let Some(source) = dependency.hosted_url.as_deref()
        && matches!(source, "git" | "local" | "url")
    {
        let label = match source {
            "git" => "git repository",
            "url" => "package URL",
            _ => "local package",
        };
        return Ok(fixed(dependency, label.to_owned()));
    }
    if dependency.ecosystem == Cran
        && let Some(source) = dependency.hosted_url.as_deref()
    {
        let label = match source {
            "git" => "git repository".to_owned(),
            "local" => "local package".to_owned(),
            "url" => "package URL".to_owned(),
            other => other.to_owned(),
        };
        return Ok(fixed(dependency, label));
    }
    if is_go_exclude(&dependency) {
        return Ok(fixed(dependency, "excluded version".to_owned()));
    }
    if dependency.hosted_url.as_deref() == Some("path")
        && let Some(path) = local_dependency_path(&dependency, document_uri)
    {
        return Ok(local_directory_suggestion(dependency, path));
    }
    if is_registry_dependency(
        dependency.ecosystem,
        &dependency.name,
        &dependency.requirement,
    ) {
        return Err(crate::boxed(dependency));
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
    if dependency.hosted_url.as_deref() == Some("hg") {
        return Ok(fixed(dependency, "hg repository".to_owned()));
    }
    if is_composer_platform(&dependency) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }
    if is_fixed_spec(&dependency.requirement) {
        let requirement = dependency.requirement.as_str().to_owned();
        return Ok(fixed(dependency, requirement));
    }

    Err(crate::boxed(dependency))
}

pub(crate) fn deno_import_has_no_suggestions(dependency: &Dependency) -> bool {
    dependency.ecosystem == Deno
        && !dependency.requirement.starts_with("jsr:")
        && !dependency.requirement.starts_with("npm:")
        && !dependency.requirement_prefix.starts_with("jsr:")
        && !dependency.requirement_prefix.starts_with("npm:")
}

fn local_directory_suggestion(dependency: Dependency, path: LocalDependencyPath) -> Suggestion {
    let LocalDependencyPath { display, resolved } = path;
    if crate::path(&resolved).exists() {
        return directory(dependency, display, resolved);
    }
    directory_not_found(dependency, display)
}

fn is_composer_platform(dependency: &Dependency) -> bool {
    dependency.ecosystem == Composer && is_composer_platform_dependency(&dependency.name)
}

fn is_composer_branch_alias_dependency(dependency: &Dependency) -> bool {
    dependency.ecosystem == Composer
        && (dependency.requirement.trim().starts_with("dev-")
            || dependency
                .requirement_suffix
                .trim_start()
                .starts_with("as "))
}

fn is_npm_name_only_metadata(dependency: &Dependency) -> bool {
    dependency.ecosystem == Npm
        && dependency.requirement.is_empty()
        && matches!(
            dependency.group.as_str(),
            "bundledDependencies" | "bundleDependencies" | "trustedDependencies"
        )
}

fn is_paket_reference(dependency: &Dependency) -> bool {
    dependency.ecosystem == Dotnet && dependency.group == "paket.references"
}

fn is_go_exclude(dependency: &Dependency) -> bool {
    dependency.ecosystem == Go && dependency.group == "exclude"
}

fn is_npm_bare_local_path(dependency: &Dependency) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Npm
        && (requirement.starts_with('/')
            || requirement.starts_with("./")
            || requirement.starts_with("../")
            || requirement.starts_with("~/"))
}

fn is_npm_override_reference(dependency: &Dependency) -> bool {
    dependency.ecosystem == Npm
        && matches!(dependency.group.as_str(), "overrides" | "pnpm.overrides")
        && dependency
            .requirement
            .trim()
            .strip_prefix('$')
            .is_some_and(|name| {
                !name.is_empty()
                    && !name.chars().any(|ch| ch.is_ascii_whitespace())
                    && !name.starts_with('/')
                    && !name.starts_with('.')
            })
}

fn is_npm_unsupported_git_dependency(dependency: &Dependency) -> bool {
    dependency.ecosystem == Npm && is_unsupported_npm_git_requirement(&dependency.requirement)
}

fn is_npm_unsupported_protocol_dependency(dependency: &Dependency) -> bool {
    let requirement = dependency.requirement.trim();
    dependency.ecosystem == Npm
        && (requirement.starts_with("exec:") || requirement.starts_with("patch:"))
}

fn is_docker_build_path(dependency: &Dependency) -> bool {
    dependency.ecosystem == Docker && dependency.group == "services.build"
}

fn is_docker_argument_reference(dependency: &Dependency) -> bool {
    dependency.ecosystem == Docker
        && (dependency.name.contains('$') || dependency.requirement.contains('$'))
}

fn is_unsupported_dotnet_version(dependency: &Dependency) -> bool {
    dependency.ecosystem == Dotnet && is_unsupported_dotnet_requirement(&dependency.requirement)
}
