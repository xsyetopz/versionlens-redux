use crate::positions::offset_range;
use std::iter;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Cpp;
const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

type CppDependencies = Vec<Dependency>;

#[derive(Clone, Copy)]
struct Quoted<'a> {
    value: &'a str,
    start: usize,
    end: usize,
}

#[derive(Clone, Copy)]
struct Field<'a> {
    value: &'a str,
    start: usize,
    end: usize,
}

pub(crate) fn parse_xmake_lua(text: &str) -> CppDependencies {
    let mut dependencies = vec![];
    for call in function_calls(text, "add_requires", &["--"]) {
        for quoted in quoted_strings(call.value, call.start) {
            if let Some(dependency) = xmake_dependency(text, quoted) {
                dependencies.push(dependency);
            }
        }
    }
    dependencies
}

pub(crate) fn parse_cmake(text: &str) -> CppDependencies {
    let mut dependencies = vec![];
    for function in [
        "FetchContent_Declare",
        "ExternalProject_Add",
        "CPMAddPackage",
    ] {
        for call in function_calls(text, function, &["#"]) {
            dependencies.extend(cmake_call_dependencies(text, function, call));
        }
    }
    dependencies
}

pub(crate) fn parse_meson_wrap(text: &str) -> CppDependencies {
    let mut dependencies = vec![];
    let mut url: Option<Field<'_>> = None;
    let mut revision: Option<Field<'_>> = None;
    let mut directory: Option<Field<'_>> = None;
    let mut offset = 0usize;
    for line in text.lines() {
        let trimmed = line.trim();
        let leading = line.len() - line.trim_start().len();
        if let Some((key, value)) = trimmed.split_once('=') {
            let value = value.trim();
            let value_start = offset + leading + trimmed.find(value).unwrap_or(0);
            let field = Field {
                value,
                start: value_start,
                end: value_start + value.len(),
            };
            match key.trim() {
                "url" | "source_url" => url = Some(field),
                "revision" | "wrapdb_version" => revision = Some(field),
                "directory" => directory = Some(field),
                _ => {}
            }
        }
        offset += line.len() + 1;
    }
    let Some(url) = url else { return dependencies };
    let Some(repo) = github_repo(url.value) else {
        return dependencies;
    };
    let requirement = revision.or_else(|| github_tag_from_url(url));
    let Some(requirement) = requirement else {
        return dependencies;
    };
    dependencies.push(cpp_dependency(
        text,
        CppDependencySource {
            name: package_name(directory.map(|field| field.value).unwrap_or(repo)),
            group: "meson.wrap",
            requirement: requirement.value,
            name_span: (url.start, url.end),
            requirement_span: (requirement.start, requirement.end),
            repo: Some(repo),
        },
    ));
    dependencies
}

pub(crate) fn parse_bazel_workspace(text: &str) -> CppDependencies {
    let mut dependencies = vec![];
    for call in function_calls(text, "http_archive", &["#"]) {
        let name = field_value(call.value, call.start, "name");
        let url = first_urls_value(call.value, call.start)
            .or_else(|| field_value(call.value, call.start, "url"));
        let tag = field_value(call.value, call.start, "tag")
            .or_else(|| field_value(call.value, call.start, "version"))
            .or_else(|| url.and_then(github_tag_from_url))
            .or_else(|| field_value(call.value, call.start, "strip_prefix"));
        let (Some(name), Some(url), Some(tag)) = (name, url, tag) else {
            continue;
        };
        let Some(repo) = github_repo(url.value) else {
            continue;
        };
        dependencies.push(cpp_dependency(
            text,
            CppDependencySource {
                name: name.value,
                group: "http_archive",
                requirement: tag.value,
                name_span: (name.start, name.end),
                requirement_span: (tag.start, tag.end),
                repo: Some(repo),
            },
        ));
    }
    dependencies
}

fn xmake_dependency(text: &str, quoted: Quoted<'_>) -> Option<Dependency> {
    let value = quoted.value.trim();
    if value.is_empty() || value.contains('$') {
        return None;
    }
    let leading = quoted.value.find(value)?;
    let name_end = value
        .find(|ch: char| ch.is_whitespace())
        .unwrap_or(value.len());
    let name = &value[..name_end];
    let requirement = value[name_end..].trim();
    let value_start = quoted.start + leading;
    let (requirement, requirement_prefix) = if requirement.is_empty() {
        ("*", " ")
    } else {
        (requirement, "")
    };
    let requirement_start = if requirement == "*" {
        value_start + name.len()
    } else {
        value_start + value.find(requirement)?
    };
    let mut dependency = cpp_dependency(
        text,
        CppDependencySource {
            name,
            group: "add_requires",
            requirement,
            name_span: (value_start, value_start + name.len()),
            requirement_span: (
                requirement_start,
                requirement_start + usize::from(requirement != "*") * requirement.len(),
            ),
            repo: None,
        },
    );
    dependency.requirement_prefix = requirement_prefix.to_owned();
    Some(dependency)
}

fn cmake_call_dependencies(text: &str, function: &str, call: Field<'_>) -> CppDependencies {
    if function == "CPMAddPackage" {
        if let Some(dependency) = cpm_shorthand_dependency(text, call) {
            return vec![dependency];
        }
    }
    let first = first_call_arg(call.value, call.start);
    let name = first
        .filter(|field| !cmake_keyword(field.value))
        .or_else(|| field_value(call.value, call.start, "NAME"));
    let url = field_value(call.value, call.start, "GIT_REPOSITORY")
        .or_else(|| field_value(call.value, call.start, "URL"))
        .or_else(|| field_value(call.value, call.start, "GITHUB_REPOSITORY"));
    let tag = field_value(call.value, call.start, "GIT_TAG")
        .or_else(|| field_value(call.value, call.start, "VERSION"))
        .or_else(|| url.and_then(github_tag_from_url));
    let (Some(name), Some(url), Some(tag)) = (name, url, tag) else {
        return vec![];
    };
    let repo = github_repo(url.value).or_else(|| github_repo_from_path(url.value));
    let Some(repo) = repo else { return vec![] };
    vec![cpp_dependency(
        text,
        CppDependencySource {
            name: name.value,
            group: function,
            requirement: tag.value,
            name_span: (name.start, name.end),
            requirement_span: (tag.start, tag.end),
            repo: Some(repo),
        },
    )]
}

fn cpm_shorthand_dependency(text: &str, call: Field<'_>) -> Option<Dependency> {
    let quoted = quoted_strings(call.value, call.start).next()?;
    let spec = quoted.value;
    let rest = spec
        .strip_prefix("gh:")
        .or_else(|| spec.strip_prefix("github:"))?;
    let (repo, tag) = rest.split_once('#')?;
    let name = package_name(repo);
    let name_start = quoted.start + spec.find(name)?;
    let tag_start = quoted.start + spec.find(tag)?;
    Some(cpp_dependency(
        text,
        CppDependencySource {
            name,
            group: "CPMAddPackage",
            requirement: tag,
            name_span: (name_start, name_start + name.len()),
            requirement_span: (tag_start, tag_start + tag.len()),
            repo: Some(repo),
        },
    ))
}

struct CppDependencySource<'a> {
    name: &'a str,
    group: &'a str,
    requirement: &'a str,
    name_span: (usize, usize),
    requirement_span: (usize, usize),
    repo: Option<&'a str>,
}

fn cpp_dependency(text: &str, source: CppDependencySource<'_>) -> Dependency {
    Dependency {
        name: source.name.to_owned(),
        requirement: source.requirement.to_owned(),
        ecosystem: Cpp,
        group: source.group.to_owned(),
        hosted_url: source
            .repo
            .map(|repo| format!("{GITHUB_API_REPO_PREFIX}{repo}/tags")),
        hosted_name: source.repo.map(ToOwned::to_owned),
        range: offset_range(text, source.name_span.0, source.name_span.1),
        requirement_range: offset_range(text, source.requirement_span.0, source.requirement_span.1),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}

fn function_calls<'a>(
    text: &'a str,
    name: &'a str,
    line_comment_markers: &'a [&'a str],
) -> impl Iterator<Item = Field<'a>> + 'a {
    let mut cursor = 0usize;
    iter::from_fn(move || {
        while let Some(index) = text[cursor..].find(name) {
            let name_start = cursor + index;
            let after_name = name_start + name.len();
            if starts_in_line_comment(text, name_start, line_comment_markers) {
                cursor = after_name;
                continue;
            }
            let Some(open_offset) = text[after_name..].find('(') else {
                cursor = after_name;
                continue;
            };
            let open = after_name + open_offset;
            let Some(close) = matching_paren(text, open) else {
                cursor = after_name;
                continue;
            };
            cursor = close + 1;
            return Some(Field {
                value: &text[open + 1..close],
                start: open + 1,
                end: close,
            });
        }
        None
    })
}

fn starts_in_line_comment(text: &str, index: usize, markers: &[&str]) -> bool {
    let line_start = text[..index].rfind('\n').map_or(0, |offset| offset + 1);
    let before = &text[line_start..index];
    markers.iter().any(|marker| before.contains(marker))
}

fn cmake_keyword(value: &str) -> bool {
    matches!(
        value,
        "NAME"
            | "URL"
            | "GIT_REPOSITORY"
            | "GITHUB_REPOSITORY"
            | "GIT_TAG"
            | "VERSION"
            | "OPTIONS"
            | "DOWNLOAD_ONLY"
            | "SOURCE_DIR"
            | "BINARY_DIR"
    )
}

fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<u8> = None;
    for (offset, byte) in text.as_bytes()[open..].iter().copied().enumerate() {
        let index = open + offset;
        if let Some(current) = quote {
            if byte == current {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn quoted_strings(text: &str, base: usize) -> impl Iterator<Item = Quoted<'_>> + '_ {
    let mut cursor = 0usize;
    iter::from_fn(move || {
        let bytes = text.as_bytes();
        while cursor < bytes.len() {
            let quote = bytes[cursor];
            if quote != b'\'' && quote != b'"' {
                cursor += 1;
                continue;
            }
            let start = cursor + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != quote {
                end += 1;
            }
            cursor = end.saturating_add(1);
            if end <= bytes.len() {
                return Some(Quoted {
                    value: &text[start..end],
                    start: base + start,
                    end: base + end,
                });
            }
        }
        None
    })
}

fn field_value<'a>(text: &'a str, base: usize, key: &str) -> Option<Field<'a>> {
    let key_start = field_key_start(text, key)?;
    let after_key = key_start + key.len();
    let rest = &text[after_key..];
    let value_start = rest
        .find(|ch: char| !ch.is_whitespace() && ch != '=')
        .map(|offset| after_key + offset)?;
    let first = text.as_bytes()[value_start];
    if first == b'\'' || first == b'"' {
        let value = quoted_strings(&text[value_start..], base + value_start).next()?;
        return Some(Field {
            value: value.value,
            start: value.start,
            end: value.end,
        });
    }
    let value_end = text[value_start..]
        .find(|ch: char| ch.is_whitespace() || ch == ')' || ch == ',')
        .map_or(text.len(), |offset| value_start + offset);
    Some(Field {
        value: &text[value_start..value_end],
        start: base + value_start,
        end: base + value_end,
    })
}

fn field_key_start(text: &str, key: &str) -> Option<usize> {
    let mut cursor = 0usize;
    while let Some(offset) = text[cursor..].find(key) {
        let index = cursor + offset;
        let before_ok = index == 0
            || !text.as_bytes()[index - 1].is_ascii_alphanumeric()
                && text.as_bytes()[index - 1] != b'_';
        let after = index + key.len();
        let after_ok = after >= text.len()
            || text.as_bytes()[after].is_ascii_whitespace()
            || text.as_bytes()[after] == b'=';
        if before_ok && after_ok {
            return Some(index);
        }
        cursor = after;
    }
    None
}

fn first_call_arg(text: &str, base: usize) -> Option<Field<'_>> {
    let trimmed = text.trim_start();
    let start = text.len() - trimmed.len();
    if trimmed.starts_with('"') || trimmed.starts_with('\'') {
        let quoted = quoted_strings(trimmed, base + start).next()?;
        return Some(Field {
            value: quoted.value,
            start: quoted.start,
            end: quoted.end,
        });
    }
    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || ch == ')')
        .unwrap_or(trimmed.len());
    Some(Field {
        value: &trimmed[..end],
        start: base + start,
        end: base + start + end,
    })
}

fn first_urls_value(text: &str, base: usize) -> Option<Field<'_>> {
    let key_start = text.find("urls")?;
    let after_key = key_start + "urls".len();
    quoted_strings(&text[after_key..], base + after_key)
        .next()
        .map(|quoted| Field {
            value: quoted.value,
            start: quoted.start,
            end: quoted.end,
        })
}

fn github_repo(url: &str) -> Option<&str> {
    let rest = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .or_else(|| url.strip_prefix("git@github.com:"))?;
    github_repo_from_path(rest)
}

fn github_repo_from_path(path: &str) -> Option<&str> {
    let repo = path
        .split_once("/archive/")
        .map_or(path, |(repo, _)| repo)
        .split_once("/releases/")
        .map_or(path, |(repo, _)| repo)
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let mut parts = repo.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    (!owner.is_empty() && !name.is_empty()).then_some(&repo[..owner.len() + 1 + name.len()])
}

fn github_tag_from_url(url: Field<'_>) -> Option<Field<'_>> {
    for marker in ["/refs/tags/", "/download/", "/archive/"] {
        if let Some(index) = url.value.find(marker) {
            let start = index + marker.len();
            let mut end = url.value[start..]
                .find(['/', '?', '#'])
                .map_or(url.value.len(), |offset| start + offset);
            let mut value = &url.value[start..end];
            for suffix in [".tar.gz", ".tgz", ".zip"] {
                if let Some(stripped) = value.strip_suffix(suffix) {
                    end -= suffix.len();
                    value = stripped;
                    break;
                }
            }
            return Some(Field {
                value,
                start: url.start + start,
                end: url.start + end,
            });
        }
    }
    None
}

fn package_name(value: &str) -> &str {
    value.rsplit('/').next().unwrap_or(value)
}
