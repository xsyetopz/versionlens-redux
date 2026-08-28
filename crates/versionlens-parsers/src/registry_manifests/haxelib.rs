use crate::support;
use jsonc_parser::ast::Object;
use jsonc_parser::ast::Value::StringLit as JsonValueStringLit;
use jsonc_parser::errors::ParseError as JsonParseError;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Haxelib;

const DEFAULT_DEPENDENCY_PATHS: &[&str] = &["dependencies"];

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
    Ok(support::try_with_json_object(text, |root| {
        let mut dependencies = vec![];
        if paths.contains(&"dependencies")
            && let Some(jsonc_parser::ast::Value::Object(object)) =
                root.get("dependencies").map(|prop| &prop.value)
        {
            collect_dependencies(text, object, &mut dependencies);
        }
        dependencies
    })?
    .unwrap_or_default())
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
            support::DependencyParts {
                requirement: version.value.as_ref().to_owned(),
                name_start,
                name_end,
                requirement_start: support::string_content_span(
                    version.range.start,
                    version.range.end,
                )
                .start,
                requirement_end: support::string_content_span(
                    version.range.start,
                    version.range.end,
                )
                .end,
            },
        ));
    }
}

fn dependency(text: &str, name: &str, parts: support::DependencyParts) -> Dependency {
    let hosted_url = parts.requirement.is_empty().then(|| "latest".to_owned());
    let mut dependency = support::dependency(text, Haxelib, "dependencies", name, parts);
    dependency.hosted_url = hosted_url;
    dependency
}

fn property_name_range(text: &str, name: &str, before: usize) -> Option<(usize, usize)> {
    let key = format!("\"{name}\"");
    let key_start = text[..before].rfind(&key)?;
    Some((key_start + 1, key_start + 1 + name.len()))
}
