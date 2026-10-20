pub(crate) use super::detect::looks_like_package_json;
use crate::json_manifest::deno::parse_deno_imports;
use crate::json_manifest::dependency::{JsonDependencySource, json_manifest_string_dependency};
use crate::json_manifest::parse::parse_json_manifest;
use jsonc_parser::ast::Value::Object as JsonValueObject;
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::Deno;
use versionlens_model::Ecosystem::Npm;
use versionlens_model::Ecosystem::{Composer, Dotnet, Dub};

pub(crate) fn parse_deno_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    let mut dependencies = parse_jsr_json_project_version(text, paths).unwrap_or_default();
    dependencies.extend(parse_deno_imports(text, paths).unwrap_or_default());
    dependencies
}

pub(crate) fn parse_jsr_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_jsr_json_project_version(text, paths).unwrap_or_default()
}

fn parse_jsr_json_project_version(
    text: &str,
    dependency_paths: &[&str],
) -> Result<Vec<Dependency>, JsonParseError> {
    if !dependency_paths.contains(&"version") {
        return Ok(vec![]);
    }

    let parse_result = parse_to_ast(text, &crate::default(), &crate::default())?;
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Ok(vec![]);
    };
    let Some(name) = root.get_string("name") else {
        return Ok(vec![]);
    };
    let Some(version) = root.get_string("version") else {
        return Ok(vec![]);
    };

    Ok(vec![json_manifest_string_dependency(
        &JsonDependencySource {
            text,
            group: "version",
            ecosystem: Deno,
        },
        name,
        version,
    )])
}

type JsonManifestDependencies = Vec<Dependency>;

pub(crate) fn parse_composer_json_with_paths(
    text: &str,
    paths: &[&str],
) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Composer)
}

pub(crate) fn parse_dotnet_project_json_with_paths(
    text: &str,
    paths: &[&str],
) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Dotnet)
}

pub(crate) fn parse_dub_json_with_paths(text: &str, paths: &[&str]) -> JsonManifestDependencies {
    parse_manifest_with_paths(text, paths, Dub)
}

pub(super) fn parse_manifest_with_paths(
    text: &str,
    paths: &[&str],
    ecosystem: Ecosystem,
) -> JsonManifestDependencies {
    parse_json_manifest(text, paths, ecosystem).unwrap_or_default()
}

pub(crate) fn parse_package_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_manifest_with_paths(text, paths, Npm)
}
