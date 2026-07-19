use crate::json_manifest::paths::NPM_DEPENDENCY_PATHS;
use versionlens_model::Dependency;

use super::super::detect::looks_like_package_json as detect_package_json;
use super::generic::parse_manifest_with_paths;
use versionlens_model::Ecosystem::Npm;

pub(crate) fn parse_package_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, NPM_DEPENDENCY_PATHS, Npm)
}

pub(crate) fn looks_like_package_json(text: &str) -> bool {
    detect_package_json(text)
}
