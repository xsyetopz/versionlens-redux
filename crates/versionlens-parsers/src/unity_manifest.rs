use crate::positions::offset_range;
use jsonc_parser::ast::ObjectPropName::{String as ObjectPropString, Word as ObjectPropWord};
use jsonc_parser::ast::Value::{
    Array as JsonValueArray, Object as JsonValueObject, StringLit as JsonValueStringLit,
};
use jsonc_parser::ast::{Array, Object, ObjectProp, Value};
use jsonc_parser::parse_to_ast;

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Unity;

type UnityJsonObject<'a> = &'a Object<'a>;
type UnityJsonProperty<'a> = &'a ObjectProp<'a>;

pub(crate) fn parse_unity_project_manifest_json_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    if !dependency_paths.is_empty() && !dependency_paths.contains(&"dependencies") {
        return vec![];
    }

    let Ok(parse_result) = parse_to_ast(text, &crate::default(), &crate::default()) else {
        return vec![];
    };
    let Some(JsonValueObject(root)) = parse_result.value else {
        return vec![];
    };

    let scoped_registries = scoped_registries(&root);
    let Some(JsonValueObject(dependencies)) = property_value(&root, "dependencies") else {
        return vec![];
    };

    dependencies
        .properties
        .iter()
        .filter_map(|prop| dependency_from_property(text, prop, &scoped_registries))
        .collect()
}

#[derive(Debug)]
struct ScopedRegistry {
    url: String,
    scopes: Vec<String>,
}

fn scoped_registries(root: UnityJsonObject<'_>) -> Vec<ScopedRegistry> {
    let Some(JsonValueArray(registries)) = property_value(root, "scopedRegistries") else {
        return vec![];
    };

    registries
        .elements
        .iter()
        .filter_map(|entry| match entry {
            JsonValueObject(registry) => scoped_registry(registry),
            _ => None,
        })
        .collect()
}

fn scoped_registry(registry: UnityJsonObject<'_>) -> Option<ScopedRegistry> {
    let url = property_string_value(registry, "url")?.trim().to_owned();
    if url.is_empty() {
        return None;
    }
    let scopes = property_array(registry, "scopes")?
        .elements
        .iter()
        .filter_map(|entry| match entry {
            JsonValueStringLit(scope) => {
                let scope = scope.value.as_ref().trim();
                (!scope.is_empty()).then(|| scope.to_owned())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    (!scopes.is_empty()).then_some(ScopedRegistry { url, scopes })
}

fn dependency_from_property(
    text: &str,
    prop: UnityJsonProperty<'_>,
    scoped_registries: &[ScopedRegistry],
) -> Option<Dependency> {
    let name = prop.name.as_str();
    let JsonValueStringLit(requirement) = &prop.value else {
        return None;
    };
    let requirement_value = requirement.value.as_ref().to_string();
    let (name_start, name_end) = property_name_range(prop);
    let requirement_start = string_content_start(requirement.range.start, requirement.range.end);
    let requirement_end = string_content_end(requirement.range.start, requirement.range.end);
    let hosted_url = unity_hosted_url(name, &requirement_value, scoped_registries);

    Some(Dependency {
        name: name.to_owned(),
        requirement: requirement_value,
        ecosystem: Unity,
        group: "dependencies".to_owned(),
        hosted_url,
        hosted_name: None,
        range: offset_range(text, name_start, name_end),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn unity_hosted_url(
    name: &str,
    requirement: &str,
    scoped_registries: &[ScopedRegistry],
) -> Option<String> {
    let requirement = requirement.trim();
    if requirement.starts_with("file:") {
        return Some("file".to_owned());
    }
    if requirement.starts_with("git+")
        || requirement.starts_with("git@")
        || crate::path(requirement)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("git"))
        || requirement.contains(".git#")
    {
        return Some("git".to_owned());
    }
    if requirement.contains("://") {
        return Some("url".to_owned());
    }

    scoped_registries
        .iter()
        .flat_map(|registry| registry.scopes.iter().map(move |scope| (registry, scope)))
        .filter(|(_, scope)| unity_scope_matches(name, scope))
        .max_by_key(|(_, scope)| scope.len())
        .map(|(registry, _)| registry.url.as_str().to_owned())
}

fn unity_scope_matches(name: &str, scope: &str) -> bool {
    name == scope
        || name
            .strip_prefix(scope)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn property_value<'a>(object: &'a Object<'a>, name: &str) -> Option<&'a Value<'a>> {
    object
        .properties
        .iter()
        .find(|prop| prop.name.as_str() == name)
        .map(|prop| &prop.value)
}

fn property_string_value<'a>(object: &'a Object<'a>, name: &str) -> Option<&'a str> {
    let JsonValueStringLit(value) = property_value(object, name)? else {
        return None;
    };
    Some(value.value.as_ref())
}

fn property_array<'a>(object: &'a Object<'a>, name: &str) -> Option<&'a Array<'a>> {
    let JsonValueArray(value) = property_value(object, name)? else {
        return None;
    };
    Some(value)
}

fn property_name_range(prop: UnityJsonProperty<'_>) -> (usize, usize) {
    match &prop.name {
        ObjectPropString(lit) => (
            string_content_start(lit.range.start, lit.range.end),
            string_content_end(lit.range.start, lit.range.end),
        ),
        ObjectPropWord(lit) => (lit.range.start, lit.range.end),
    }
}

fn string_content_start(start: usize, end: usize) -> usize {
    start + usize::from(end > start)
}

fn string_content_end(start: usize, end: usize) -> usize {
    end.saturating_sub(usize::from(end > start))
}
