use crate::json_manifest::deno::parse_deno_imports;
use crate::json_manifest::dependency::{JsonDependencySource, json_manifest_string_dependency};
use crate::json_manifest::paths::{DENO_DEPENDENCY_PATHS, JSR_DEPENDENCY_PATHS, dependency_paths};
use jsonc_parser::ast::Value::Object as JsonValueObject;
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Deno;

pub(crate) fn parse_deno_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    let dependency_paths = dependency_paths(paths, DENO_DEPENDENCY_PATHS);
    let mut dependencies =
        parse_jsr_json_project_version(text, dependency_paths).unwrap_or_default();
    dependencies.extend(parse_deno_imports(text, dependency_paths).unwrap_or_default());
    dependencies
}

pub(crate) fn parse_jsr_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_jsr_json_project_version(text, dependency_paths(paths, JSR_DEPENDENCY_PATHS))
        .unwrap_or_default()
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
