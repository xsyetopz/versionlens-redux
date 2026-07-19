use jsonc_parser::ast::Object;
use jsonc_parser::ast::Value::{Object as JsonValueObject, StringLit as JsonValueStringLit};
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Haxelib;

const DEFAULT_DEPENDENCY_PATHS: &[&str] = &["dependencies"];

struct HaxelibDependencyParts {
    requirement: String,
    name_start: usize,
    name_end: usize,
    requirement_start: usize,
    requirement_end: usize,
}

pub(crate) fn parse_haxelib_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_haxelib_json(text, dependency_paths(paths)).unwrap_or_default()
}

fn dependency_paths<'a>(paths: &'a [&'a str]) -> &'a [&'a str] {
    if paths.is_empty() {
        DEFAULT_DEPENDENCY_PATHS
    } else {
        paths
    }
}

fn parse_haxelib_json(text: &str, paths: &[&str]) -> Result<Vec<Dependency>, JsonParseError> {
    let parse_result = parse_to_ast(text, &crate::default(), &crate::default())?;
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Ok(vec![]);
    };

    let mut dependencies = vec![];
    if paths.contains(&"dependencies")
        && let Some(JsonValueObject(object)) = root.get("dependencies").map(|prop| &prop.value)
    {
        collect_dependencies(text, object, &mut dependencies);
    }

    Ok(dependencies)
}

fn collect_dependencies(text: &str, object: &Object<'_>, out: &mut Vec<Dependency>) {
    for property in &object.properties {
        let JsonValueStringLit(version) = &property.value else {
            continue;
        };
        let name = property.name.as_str();
        let (name_start, name_end) = property_name_range(text, name, version.range.start)
            .unwrap_or((version.range.start, version.range.start));
        out.push(dependency(
            text,
            name,
            HaxelibDependencyParts {
                requirement: version.value.as_ref().to_owned(),
                name_start,
                name_end,
                requirement_start: string_content_start(version.range.start, version.range.end),
                requirement_end: string_content_end(version.range.start, version.range.end),
            },
        ));
    }
}

fn dependency(text: &str, name: &str, parts: HaxelibDependencyParts) -> Dependency {
    let hosted_url = parts.requirement.is_empty().then(|| "latest".to_owned());
    Dependency {
        name: name.to_owned(),
        requirement: parts.requirement,
        ecosystem: Haxelib,
        group: "dependencies".to_owned(),
        hosted_url,
        hosted_name: None,
        range: offset_range(text, parts.name_start, parts.name_end),
        requirement_range: offset_range(text, parts.requirement_start, parts.requirement_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn string_content_start(start: usize, end: usize) -> usize {
    start + usize::from(end > start)
}

fn string_content_end(start: usize, end: usize) -> usize {
    end.saturating_sub(usize::from(end > start))
}

fn property_name_range(text: &str, name: &str, before: usize) -> Option<(usize, usize)> {
    let key = format!("\"{name}\"");
    let key_start = text[..before].rfind(&key)?;
    Some((key_start + 1, key_start + 1 + name.len()))
}
