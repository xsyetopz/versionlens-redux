use crate::model::{Dependency, Ecosystem};

use super::generic::parse_manifest_with_paths;
use crate::json_manifest::{
    detect::looks_like_package_json as detect_package_json, paths::NPM_DEPENDENCY_PATHS,
};

pub(crate) fn parse_package_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, NPM_DEPENDENCY_PATHS, Ecosystem::Npm)
}

pub(crate) fn looks_like_package_json(text: &str) -> bool {
    detect_package_json(text)
}
