use jsonc_parser::ast::Value::{
    Array as JsonValueArray, Object as JsonValueObject, StringLit as JsonValueStringLit,
};
use jsonc_parser::ast::{Array, Object, StringLit};
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;

use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Vcpkg;

type VcpkgDependencies = Vec<Dependency>;
type VcpkgObject<'a> = &'a Object<'a>;

const DEFAULT_DEPENDENCY_PATHS: &[&str] = &["dependencies", "features.*.dependencies", "overrides"];

struct VcpkgDependencyParts {
    requirement: String,
    name_start: usize,
    name_end: usize,
    requirement_start: usize,
    requirement_end: usize,
}

pub(crate) fn parse_vcpkg_json_with_paths(text: &str, paths: &[&str]) -> VcpkgDependencies {
    parse_vcpkg_json(text, dependency_paths(paths)).unwrap_or_default()
}

fn dependency_paths<'a>(paths: &'a [&'a str]) -> &'a [&'a str] {
    if paths.is_empty() {
        DEFAULT_DEPENDENCY_PATHS
    } else {
        paths
    }
}

fn parse_vcpkg_json(text: &str, paths: &[&str]) -> Result<VcpkgDependencies, JsonParseError> {
    let parse_result = parse_to_ast(text, &crate::default(), &crate::default())?;
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Ok(vec![]);
    };

    let mut dependencies = vec![];
    if paths.contains(&"dependencies")
        && let Some(JsonValueArray(array)) = root.get("dependencies").map(|prop| &prop.value)
    {
        collect_dependency_array(text, "dependencies", array, &mut dependencies);
    }
    if paths.contains(&"features.*.dependencies")
        && let Some(features) = root.get_object("features")
    {
        collect_feature_dependencies(text, features, &mut dependencies);
    }
    if paths.contains(&"overrides")
        && let Some(JsonValueArray(array)) = root.get("overrides").map(|prop| &prop.value)
    {
        collect_dependency_array(text, "overrides", array, &mut dependencies);
    }

    Ok(dependencies)
}

fn collect_feature_dependencies(
    text: &str,
    features: VcpkgObject<'_>,
    out: &mut VcpkgDependencies,
) {
    for feature in &features.properties {
        let JsonValueObject(feature_object) = &feature.value else {
            continue;
        };
        let Some(JsonValueArray(array)) =
            feature_object.get("dependencies").map(|prop| &prop.value)
        else {
            continue;
        };
        let group = format!("features.{}.dependencies", feature.name.as_str());
        collect_dependency_array(text, &group, array, out);
    }
}

fn collect_dependency_array(
    text: &str,
    group: &str,
    array: &Array<'_>,
    out: &mut VcpkgDependencies,
) {
    for element in &array.elements {
        match element {
            JsonValueStringLit(lit) => out.push(name_only_dependency(text, group, lit)),
            JsonValueObject(object) => {
                if let Some(dependency) = object_dependency(text, group, object) {
                    out.push(dependency);
                }
            }
            _ => {}
        }
    }
}

fn name_only_dependency(text: &str, group: &str, lit: &StringLit<'_>) -> Dependency {
    let name_start = string_content_start(lit.range.start, lit.range.end);
    let name_end = string_content_end(lit.range.start, lit.range.end);
    let mut dependency = dependency(
        text,
        group,
        lit.value.as_ref(),
        VcpkgDependencyParts {
            requirement: "".to_owned(),
            name_start,
            name_end,
            requirement_start: name_end,
            requirement_end: name_end,
        },
    );
    dependency.hosted_url = Some("baseline".to_owned());
    dependency
}

fn object_dependency(text: &str, group: &str, object: VcpkgObject<'_>) -> Option<Dependency> {
    let name = object.get_string("name")?;
    let name_start = string_content_start(name.range.start, name.range.end);
    let name_end = string_content_end(name.range.start, name.range.end);

    let version = if group == "overrides" {
        object.get_string("version")
    } else {
        object.get_string("version>=")
    };

    let Some(version) = version else {
        let mut dependency = dependency(
            text,
            group,
            name.value.as_ref(),
            VcpkgDependencyParts {
                requirement: "".to_owned(),
                name_start,
                name_end,
                requirement_start: name_end,
                requirement_end: name_end,
            },
        );
        dependency.hosted_url = Some("baseline".to_owned());
        return Some(dependency);
    };

    Some(dependency(
        text,
        group,
        name.value.as_ref(),
        VcpkgDependencyParts {
            requirement: version.value.as_ref().to_owned(),
            name_start,
            name_end,
            requirement_start: string_content_start(version.range.start, version.range.end),
            requirement_end: string_content_end(version.range.start, version.range.end),
        },
    ))
}

fn dependency(text: &str, group: &str, name: &str, parts: VcpkgDependencyParts) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: parts.requirement,
        ecosystem: Vcpkg,
        group: group.to_owned(),
        hosted_url: None,
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
