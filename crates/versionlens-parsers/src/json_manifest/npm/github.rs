use jsonc_parser::ast::ObjectProp;
use jsonc_parser::common::Ranged;

use crate::model::Dependency;

use super::super::dependency::{
    JsonDependencyRanges, JsonDependencySource, json_manifest_dependency,
};

mod reference;

use reference::{GitHubReferenceKind, github_reference, is_commit_sha};

pub(in crate::json_manifest) fn github_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    value: &str,
    value_start: usize,
) -> Option<Dependency> {
    let reference = github_reference(value)?;
    if reference.fragment.is_none() && reference.kind != GitHubReferenceKind::Shortcut {
        return None;
    }
    if reference.fragment.is_some_and(str::is_empty) {
        return None;
    }
    let requirement = reference
        .fragment
        .and_then(|fragment| fragment.strip_prefix("semver:").or(Some(fragment)))
        .unwrap_or_default();

    let mut dependency = json_manifest_dependency(
        source,
        reference.name,
        requirement.to_owned(),
        JsonDependencyRanges {
            name_start: prop.name.range().start,
            name_end: prop.name.range().end,
            requirement_start: value_start,
            requirement_end: value_start + value.len(),
        },
    );
    dependency.requirement_prefix =
        requirement_prefix(value, requirement, reference.fragment.is_none());
    dependency.hosted_url = Some(format!(
        "https://api.github.com/repos/{}/{}",
        reference.name,
        github_api_path(reference.fragment, requirement)
    ));
    Some(dependency)
}

fn requirement_prefix(value: &str, requirement: &str, missing_fragment: bool) -> String {
    if missing_fragment {
        return format!("{value}#");
    }
    value[..value.len() - requirement.len()].to_owned()
}

fn github_api_path(fragment: Option<&str>, requirement: &str) -> &'static str {
    if is_commit_sha(requirement) {
        return "commits";
    }
    if fragment.is_some_and(|fragment| fragment.starts_with("semver:"))
        || looks_like_version_requirement(requirement)
    {
        return "tags";
    }
    "commits"
}

fn looks_like_version_requirement(requirement: &str) -> bool {
    requirement
        .trim_start_matches(['v', 'V'])
        .bytes()
        .next()
        .is_some_and(|byte| {
            byte.is_ascii_digit() || matches!(byte, b'^' | b'~' | b'<' | b'>' | b'=' | b'*')
        })
}
