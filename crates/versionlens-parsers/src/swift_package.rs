use crate::github;
use crate::positions::offset_range;
use crate::quoted::{QuotedString, double_quoted_string_at};
use crate::support;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Swift;

pub(crate) fn parse_swift_package(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut search_start = 0;
    while let Some(relative_start) = text[search_start..].find(".package(") {
        let start = search_start + relative_start;
        let end = support::balanced_delimited_end(text, start, b'(', b')')
            .unwrap_or_else(|| line_end(text, start));
        if let Some(dependency) = parse_package_call(text, start, end) {
            dependencies.push(dependency);
        }
        search_start = end;
    }
    dependencies
}

fn line_end(text: &str, start: usize) -> usize {
    text[start..]
        .find('\n')
        .map_or(text.len(), |relative| start + relative)
}

fn parse_package_call(text: &str, start: usize, end: usize) -> Option<Dependency> {
    let call = &text[start..end];
    if let Some(path) = find_labeled_string(text, start, call, "path:") {
        let name = package_name_from_path(path.value);
        let mut dependency = swift_dependency(text, "dependencies", &name, path.value, path);
        dependency.hosted_url = Some("path".to_owned());
        return Some(dependency);
    }

    let url = find_labeled_string(text, start, call, "url:")?;
    let explicit_name = find_labeled_string(text, start, call, "name:");
    let name = explicit_name
        .map(|name| name.value.to_owned())
        .unwrap_or_else(|| package_name_from_url(url.value));

    let requirement = find_labeled_string(text, start, call, "from:")
        .or_else(|| find_labeled_string(text, start, call, "exact:"))
        .or_else(|| find_function_string(text, start, call, ".exact("))
        .or_else(|| find_labeled_string_after(text, start, call, ".upToNextMajor(", "from:"))
        .or_else(|| find_labeled_string_after(text, start, call, ".upToNextMinor(", "from:"));

    if let Some(requirement) = requirement {
        let mut dependency =
            swift_dependency(text, "dependencies", &name, requirement.value, requirement);
        if let Some(repo) = github::repository_from_url(
            url.value,
            &[
                "https://github.com/",
                "http://github.com/",
                "git@github.com:",
            ],
        ) {
            dependency.hosted_name = Some(repo.to_owned());
            dependency.hosted_url = Some(github::tags_url(repo)?);
        } else {
            dependency.hosted_url = Some("git".to_owned());
        }
        return Some(dependency);
    }

    let branch = find_labeled_string(text, start, call, "branch:");
    let revision = find_labeled_string(text, start, call, "revision:")
        .or_else(|| find_function_string(text, start, call, ".revision("));
    if let Some(requirement) = branch.or(revision) {
        let mut dependency =
            swift_dependency(text, "dependencies", &name, requirement.value, requirement);
        dependency.hosted_url = Some("git".to_owned());
        return Some(dependency);
    }

    None
}

fn find_labeled_string<'a>(
    text: &'a str,
    call_start: usize,
    call: &'a str,
    label: &str,
) -> Option<QuotedString<'a>> {
    find_labeled_string_after(text, call_start, call, "", label)
}

fn find_labeled_string_after<'a>(
    text: &'a str,
    call_start: usize,
    call: &'a str,
    prefix: &str,
    label: &str,
) -> Option<QuotedString<'a>> {
    let prefix_start = if prefix.is_empty() {
        0
    } else {
        call.find(prefix)?
    };
    let label_start = prefix_start + call[prefix_start..].find(label)?;
    let quote_start = label_start + call[label_start..].find('"')?;
    double_quoted_string_at(text, call_start + quote_start)
}

fn find_function_string<'a>(
    text: &'a str,
    call_start: usize,
    call: &'a str,
    function: &str,
) -> Option<QuotedString<'a>> {
    let function_start = call.find(function)?;
    let quote_start = function_start + call[function_start..].find('"')?;
    double_quoted_string_at(text, call_start + quote_start)
}

fn swift_dependency(
    text: &str,
    group: &str,
    name: &str,
    requirement: &str,
    value: QuotedString<'_>,
) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Swift,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, value.start, value.end),
        requirement_range: offset_range(text, value.start, value.end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

fn package_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(url)
        .trim_end_matches(".git")
        .to_owned()
}

fn package_name_from_path(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}
