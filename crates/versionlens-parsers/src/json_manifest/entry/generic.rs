use crate::model::{Dependency, Ecosystem};

use crate::json_manifest::{
    parse::parse_json_manifest,
    paths::{
        COMPOSER_DEPENDENCY_PATHS, DOTNET_PROJECT_DEPENDENCY_PATHS, DUB_DEPENDENCY_PATHS,
        dependency_paths,
    },
};

pub(crate) fn parse_composer_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, COMPOSER_DEPENDENCY_PATHS, Ecosystem::Composer)
}

pub(crate) fn parse_dotnet_project_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(
        text,
        paths,
        DOTNET_PROJECT_DEPENDENCY_PATHS,
        Ecosystem::Dotnet,
    )
}

pub(crate) fn parse_dub_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, DUB_DEPENDENCY_PATHS, Ecosystem::Dub)
}

pub(super) fn parse_manifest_with_paths(
    text: &str,
    paths: &[&str],
    default_paths: &[&str],
    ecosystem: Ecosystem,
) -> Vec<Dependency> {
    parse_json_manifest(text, dependency_paths(paths, default_paths), ecosystem).unwrap_or_default()
}
