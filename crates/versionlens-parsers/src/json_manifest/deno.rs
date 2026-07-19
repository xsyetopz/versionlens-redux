use crate::json_manifest::npm::trim_selector;
use jsonc_parser::ast::ObjectProp;
use jsonc_parser::ast::Value::{Object as JsonValueObject, StringLit as JsonValueStringLit};
use jsonc_parser::common::Ranged;
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;

use versionlens_model::Dependency;

use super::dependency::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency, string_content_start,
};
use versionlens_model::Ecosystem::{Deno, Npm};

pub(super) fn parse_deno_imports(
    text: &str,
    dependency_paths: &[&str],
) -> Result<Vec<Dependency>, JsonParseError> {
    let parse_result = parse_to_ast(text, &crate::default(), &crate::default())?;
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Ok(vec![]);
    };
    let mut dependencies = vec![];
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
            let JsonValueObject(imports) = &scope.value else {
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
    let JsonValueStringLit(lit) = &prop.value else {
        return None;
    };
    let raw = lit.value.as_ref();
    if raw.starts_with("catalog:") || raw.starts_with("workspace:") {
        return None;
    }
    let ecosystem = if raw.starts_with("npm:") { Npm } else { Deno };
    let requirement_start = string_content_start(lit.range.start, lit.range.end);

    let requirement_parts = deno_import_requirement(raw);
    let requirement = requirement_parts
        .as_ref()
        .map_or_else(|| raw.to_owned(), |parts| parts.requirement.to_owned());
    let requirement_is_empty = requirement_parts
        .as_ref()
        .is_some_and(|parts| parts.requirement.is_empty());
    let requirement_range_start = if requirement_is_empty {
        requirement_start + raw.len()
    } else {
        requirement_start
    };
    let requirement_end = if requirement_is_empty {
        requirement_range_start
    } else {
        requirement_start + raw.len()
    };
    let mut dependency = json_manifest_dependency(
        &JsonDependencySource {
            text,
            group,
            ecosystem,
        },
        trim_selector(prop.name.as_str()),
        requirement,
        JsonDependencyRanges {
            name_start: prop.name.range().start,
            name_end: prop.name.range().end,
            requirement_start: requirement_range_start,
            requirement_end,
        },
    );
    if let Some(parts) = requirement_parts {
        dependency.requirement_prefix = parts.prefix;
        dependency.requirement_suffix = parts.suffix.to_owned();
    }
    dependency.hosted_name = deno_registry_package_name(raw).map(|value| value.to_owned());
    Some(dependency)
}

struct DenoImportRequirement<'a> {
    prefix: String,
    requirement: &'a str,
    suffix: &'a str,
}

fn deno_import_requirement(raw: &str) -> Option<DenoImportRequirement<'_>> {
    let scheme_len = raw
        .strip_prefix("jsr:")
        .map(|_| "jsr:".len())
        .or_else(|| raw.strip_prefix("npm:").map(|_| "npm:".len()))?;
    let is_directory_specifier =
        raw.ends_with('/') && (raw.starts_with("jsr:/") || raw.starts_with("npm:/"));
    let suffix = if is_directory_specifier { "/" } else { "" };
    let spec = if is_directory_specifier {
        raw.strip_suffix('/')?
    } else {
        raw
    };
    let spec_start = if spec[scheme_len..].starts_with('/') {
        scheme_len + 1
    } else {
        scheme_len
    };
    let name = spec[spec_start..]
        .trim_start_matches('/')
        .trim_end_matches('/');
    if name.is_empty() {
        return None;
    }

    if let Some(version_at) = spec.rfind('@').filter(|index| *index > spec_start) {
        return Some(DenoImportRequirement {
            prefix: raw[..version_at + 1].to_owned(),
            requirement: &spec[version_at + 1..],
            suffix,
        });
    }

    Some(DenoImportRequirement {
        prefix: format!("{spec}@"),
        requirement: "",
        suffix,
    })
}

fn deno_registry_package_name(requirement: &str) -> Option<&str> {
    let spec = requirement
        .strip_prefix("jsr:")
        .or_else(|| requirement.strip_prefix("npm:"))?;
    let name = spec
        .rfind('@')
        .filter(|index| *index > 0)
        .map_or(spec, |index| &spec[..index]);
    let name = name.trim_start_matches('/').trim_end_matches('/');
    (!name.is_empty()).then_some(name)
}
