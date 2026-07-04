use jsonc_parser::ast::{ObjectProp, Value};
use jsonc_parser::common::Ranged;
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};

use crate::model::{Dependency, Ecosystem};

use super::dependency::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, string_content_start,
};
use super::npm;

pub(super) fn parse_deno_imports(
    text: &str,
    dependency_paths: &[&str],
) -> Result<Vec<Dependency>, jsonc_parser::errors::ParseError> {
    let parse_result = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())?;
    let Some(Value::Object(root)) = parse_result.value else {
        return Ok(Vec::new());
    };
    let mut dependencies = Vec::new();
    if dependency_paths.contains(&"imports")
        && let Some(imports) = root.get_object("imports")
    {
        dependencies.extend(
            imports
                .properties
                .iter()
                .filter_map(|prop| deno_import_dependency(text, "imports", prop)),
        );
    }
    if dependency_paths.contains(&"scopes")
        && let Some(scopes) = root.get_object("scopes")
    {
        for scope in &scopes.properties {
            let Value::Object(imports) = &scope.value else {
                continue;
            };
            let group = deno_scope_group(scope.name.as_str());
            dependencies.extend(
                imports
                    .properties
                    .iter()
                    .filter_map(|prop| deno_import_dependency(text, &group, prop)),
            );
        }
    }

    Ok(dependencies)
}

fn deno_scope_group(scope: &str) -> String {
    format!("scopes.{scope}")
}

fn deno_import_dependency(text: &str, group: &str, prop: &ObjectProp<'_>) -> Option<Dependency> {
    let Value::StringLit(lit) = &prop.value else {
        return None;
    };
    let raw = lit.value.as_ref();
    if raw.starts_with("catalog:") || raw.starts_with("workspace:") {
        return None;
    }
    let ecosystem = if raw.starts_with("npm:") {
        Ecosystem::Npm
    } else {
        Ecosystem::Deno
    };
    let requirement_start = string_content_start(lit.range.start, lit.range.end);

    let mut dependency = json_manifest_dependency(
        &JsonDependencySource {
            text,
            group,
            ecosystem,
        },
        npm::trim_selector(prop.name.as_str()),
        raw.to_owned(),
        JsonDependencyRanges {
            name_start: prop.name.range().start,
            name_end: prop.name.range().end,
            requirement_start,
            requirement_end: requirement_start + raw.len(),
        },
    );
    dependency.hosted_name = deno_registry_package_name(raw).map(str::to_owned);
    Some(dependency)
}

fn deno_registry_package_name(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let name = spec
        .rfind('@')
        .filter(|index| *index > 0)
        .map_or(spec, |index| &spec[..index]);
    (!name.is_empty()).then_some(name)
}
