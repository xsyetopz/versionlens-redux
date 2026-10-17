use crate::json_manifest::dependency::{JsonDependencySource, json_manifest_string_dependency};
use jsonc_parser::ast::Value::Object as JsonValueObject;
use jsonc_parser::parse_to_ast;
use versionlens_model::Dependency;

use super::super::detect::looks_like_package_json as detect_package_json;
use super::generic::parse_manifest_with_paths;
use versionlens_model::Ecosystem::Npm;

pub(crate) fn parse_package_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, Npm)
}

/// The root `version` field of a package.json: the project version, not a dependency.
pub(crate) fn parse_package_json_project_version(text: &str) -> Vec<Dependency> {
    let Ok(parse_result) = parse_to_ast(text, &crate::default(), &crate::default()) else {
        return Vec::new();
    };
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Vec::new();
    };
    let Some(version) = root.get_string("version") else {
        return Vec::new();
    };

    vec![json_manifest_string_dependency(
        &JsonDependencySource {
            text,
            group: "version",
            ecosystem: Npm,
        },
        version,
        version,
    )]
}

pub(crate) fn looks_like_package_json(text: &str) -> bool {
    detect_package_json(text)
}
