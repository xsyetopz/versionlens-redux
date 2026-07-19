use crate::positions::offset_range;
use crate::quoted::{QuotedString, double_quoted_string_at};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Zig;

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(crate) fn parse_zig_build_zon(text: &str) -> Vec<Dependency> {
    let Some(dependencies_start) = text.find(".dependencies") else {
        return vec![];
    };
    let Some(block_start) = text[dependencies_start..].find(".{") else {
        return vec![];
    };
    let block_start = dependencies_start + block_start;
    let block_end = zon_block_end(text, block_start).unwrap_or(text.len());
    parse_dependency_block(text, block_start, block_end)
}

fn parse_dependency_block(text: &str, start: usize, end: usize) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let mut search = start;
    while search < end {
        let Some(relative_dot) = text[search..end].find('.') else {
            break;
        };
        let name_start = search + relative_dot + 1;
        let Some((name, name_end)) = parse_identifier(text, name_start) else {
            search = name_start;
            continue;
        };
        let Some(equals) = text[name_end..end].find('=') else {
            break;
        };
        let value_start = name_end + equals + 1;
        let Some(object_relative) = text[value_start..end].find(".{") else {
            search = value_start;
            continue;
        };
        let object_start = value_start + object_relative;
        let Some(object_end) = zon_block_end(text, object_start) else {
            break;
        };
        if let Some(dependency) = parse_dependency_object(text, &name, object_start, object_end) {
            dependencies.push(dependency);
        }
        search = object_end;
    }
    dependencies
}

fn parse_dependency_object(text: &str, name: &str, start: usize, end: usize) -> Option<Dependency> {
    if let Some(path) = find_field_string(text, start, end, ".path") {
        let mut dependency = zig_dependency(text, name, path.value, path.start, path.end);
        dependency.hosted_url = Some("path".to_owned());
        return Some(dependency);
    }

    let url = find_field_string(text, start, end, ".url")?;
    let mut dependency = if let Some(version) = version_in_url(url.value) {
        zig_dependency(
            text,
            name,
            version.value,
            url.start + version.start,
            url.start + version.end,
        )
    } else if let Some(hash) = find_field_string(text, start, end, ".hash") {
        let mut dependency = zig_dependency(text, name, hash.value, hash.start, hash.end);
        dependency.hosted_url = Some("hash".to_owned());
        return Some(dependency);
    } else {
        let mut dependency = zig_dependency(text, name, url.value, url.start, url.end);
        dependency.hosted_url = Some("url".to_owned());
        return Some(dependency);
    };

    if let Some(repo) = github_repo(url.value) {
        dependency.hosted_name = Some(repo.to_owned());
        dependency.hosted_url = Some(format!("{GITHUB_API_REPO_PREFIX}{repo}/tags"));
    } else {
        dependency.hosted_url = Some("url".to_owned());
    }
    Some(dependency)
}

fn zon_block_end(text: &str, start: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = start;
    let mut depth = 0usize;
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
        } else if byte == b'.' && bytes.get(index + 1) == Some(&b'{') {
            depth += 1;
            index += 1;
        } else if byte == b'}' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(index + 1);
            }
        }
        index += 1;
    }
    None
}

fn parse_identifier(text: &str, start: usize) -> Option<(String, usize)> {
    if text.as_bytes().get(start) == Some(&b'@') {
        let string = double_quoted_string_at(text, start + 1)?;
        return Some((string.value.to_owned(), string.end + 1));
    }

    let mut end = start;
    for (offset, ch) in text[start..].char_indices() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            end = start + offset + ch.len_utf8();
        } else {
            break;
        }
    }
    (end > start).then(|| (text[start..end].to_owned(), end))
}

fn find_field_string<'a>(
    text: &'a str,
    start: usize,
    end: usize,
    field: &str,
) -> Option<QuotedString<'a>> {
    let relative = text[start..end].find(field)?;
    let field_start = start + relative;
    let equals = field_start + text[field_start..end].find('=')?;
    let quote = equals + text[equals..end].find('"')?;
    double_quoted_string_at(text, quote)
}

fn zig_dependency(
    text: &str,
    name: &str,
    requirement: &str,
    requirement_start: usize,
    requirement_end: usize,
) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Zig,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, requirement_start, requirement_end),
        requirement_range: offset_range(text, requirement_start, requirement_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

struct VersionInUrl<'a> {
    value: &'a str,
    start: usize,
    end: usize,
}

fn version_in_url(url: &str) -> Option<VersionInUrl<'_>> {
    let marker = "/archive/refs/tags/";
    if let Some(index) = url.find(marker) {
        let start = index + marker.len();
        let mut end = url[start..]
            .find(['/', '?', '#'])
            .map_or(url.len(), |relative| start + relative);
        for suffix in [".tar.gz", ".tar.xz", ".tgz", ".zip"] {
            if let Some(stripped) = url[start..end].strip_suffix(suffix) {
                end = start + stripped.len();
                break;
            }
        }
        return version_from_url_segment(url, start, end);
    }

    let marker = "/releases/download/";
    let index = url.find(marker)?;
    let start = index + marker.len();
    let end = url[start..]
        .find(['/', '?', '#'])
        .map_or(url.len(), |relative| start + relative);
    version_from_url_segment(url, start, end)
}

fn version_from_url_segment(url: &str, start: usize, end: usize) -> Option<VersionInUrl<'_>> {
    let value = &url[start..end];
    (!value.is_empty()).then_some(VersionInUrl { value, start, end })
}

fn github_repo(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))?;
    let repo = rest.split_once("/archive/").map_or(rest, |(repo, _)| repo);
    let repo = repo.split_once("/releases/").map_or(repo, |(repo, _)| repo);
    let repo = repo.trim_end_matches('/').trim_end_matches(".git");
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    (parts.next().is_none() && !owner.is_empty() && !name.is_empty()).then_some(repo)
}
