use crate::support;
use jsonc_parser::ast::Value::{
    Array as JsonValueArray, Object as JsonValueObject, StringLit as JsonValueStringLit,
};
use jsonc_parser::ast::{Array, Object, StringLit};
use jsonc_parser::errors::ParseError as JsonParseError;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Vcpkg;

type VcpkgDependencies = Vec<Dependency>;
type VcpkgObject<'a> = &'a Object<'a>;

const DEFAULT_DEPENDENCY_PATHS: &[&str] = &["dependencies", "features.*.dependencies", "overrides"];

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
    Ok(support::try_with_json_object(text, |root| {
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
        dependencies
    })?
    .unwrap_or_default())
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
    let name_span = support::string_content_span(lit.range.start, lit.range.end);
    baseline_dependency(
        text,
        group,
        lit.value.as_ref(),
        name_span.start,
        name_span.end,
    )
}

fn object_dependency(text: &str, group: &str, object: VcpkgObject<'_>) -> Option<Dependency> {
    let name = object.get_string("name")?;
    let name_span = support::string_content_span(name.range.start, name.range.end);

    let version = if group == "overrides" {
        object.get_string("version")
    } else {
        object.get_string("version>=")
    };

    let Some(version) = version else {
        return Some(baseline_dependency(
            text,
            group,
            name.value.as_ref(),
            name_span.start,
            name_span.end,
        ));
    };

    Some(support::dependency(
        text,
        Vcpkg,
        group,
        name.value.as_ref(),
        support::DependencyParts {
            requirement: version.value.as_ref().to_owned(),
            name_start: name_span.start,
            name_end: name_span.end,
            requirement_start: support::string_content_span(version.range.start, version.range.end)
                .start,
            requirement_end: support::string_content_span(version.range.start, version.range.end)
                .end,
        },
    ))
}

fn baseline_dependency(
    text: &str,
    group: &str,
    name: &str,
    name_start: usize,
    name_end: usize,
) -> Dependency {
    let mut dependency = support::dependency(
        text,
        Vcpkg,
        group,
        name,
        support::DependencyParts {
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
