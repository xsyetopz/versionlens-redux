use crate::positions::offset_range;
use crate::quoted::{QuotedString, double_quoted_string_at};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Swift;

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(crate) fn parse_swift_package(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut search_start = 0;
    while let Some(relative_start) = text[search_start..].find(".package(") {
        let start = search_start + relative_start;
        let end = package_call_end(text, start).unwrap_or_else(|| line_end(text, start));
        if let Some(dependency) = parse_package_call(text, start, end) {
            dependencies.push(dependency);
        }
        search_start = end;
    }
    dependencies
}

fn package_call_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = start;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        if byte == b'"' {
            in_string = true;
        } else if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index + 1);
            }
        }
        index += 1;
    }
    None
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
        if let Some(repo) = github_repo(url.value) {
            dependency.hosted_name = Some(repo.to_owned());
            dependency.hosted_url = Some(format!("{GITHUB_API_REPO_PREFIX}{repo}/tags"));
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

fn github_repo(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .or_else(|| url.strip_prefix("git@github.com:"))?;
    let repo = rest.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    (parts.next().is_none() && !owner.is_empty() && !name.is_empty()).then_some(repo)
}
