use std::collections::BTreeMap;

use crate::positions::offset_range;
use crate::scanner::matching_delimiter;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Bazel;

type ParsedBazelDependency = Option<Dependency>;
type BazelArguments = BTreeMap<String, ArgumentValue>;

struct FixedOverrideRequest<'a> {
    directive: &'a str,
    call_start: usize,
    close: usize,
    args: &'a BazelArguments,
    source: &'a str,
    requirement_key: &'a str,
}

pub(crate) fn parse_bazel_module_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    let mut dependencies = vec![];
    for directive in [
        "bazel_dep",
        "single_version_override",
        "multiple_version_override",
        "git_override",
        "archive_override",
        "local_path_override",
    ] {
        if !path_enabled(dependency_paths, directive) {
            continue;
        }
        collect_directive(text, directive, &mut dependencies);
    }
    dependencies.sort_by_key(|dependency| dependency.range.start.line);
    dependencies
}

fn path_enabled(paths: &[&str], directive: &str) -> bool {
    paths.is_empty() || paths.contains(&directive)
}

fn collect_directive(text: &str, directive: &str, out: &mut Vec<Dependency>) {
    let mut offset = 0usize;
    while let Some(relative) = text[offset..].find(directive) {
        let start = offset + relative;
        if !is_identifier_boundary(text, start, directive.len()) {
            offset = start + directive.len();
            continue;
        }
        let paren = skip_ws(text, start + directive.len());
        if text.as_bytes().get(paren) != Some(&b'(') {
            offset = start + directive.len();
            continue;
        }
        let Some(close) = matching_delimiter(text, paren, b'(', b')') else {
            break;
        };
        if let Some(dependency) = directive_dependency(text, directive, start, paren, close) {
            out.push(dependency);
        }
        offset = close + 1;
    }
}

fn directive_dependency(
    text: &str,
    directive: &str,
    call_start: usize,
    paren: usize,
    close: usize,
) -> ParsedBazelDependency {
    let args = parse_kwargs(text, paren + 1, close);
    match directive {
        "bazel_dep" => bazel_dep_dependency(text, call_start, close, &args),
        "single_version_override" | "multiple_version_override" => {
            version_override_dependency(text, directive, call_start, close, &args)
        }
        "git_override" => fixed_override_dependency(
            text,
            FixedOverrideRequest {
                directive,
                call_start,
                close,
                args: &args,
                source: "git",
                requirement_key: "commit",
            },
        ),
        "archive_override" => fixed_override_dependency(
            text,
            FixedOverrideRequest {
                directive,
                call_start,
                close,
                args: &args,
                source: "archive",
                requirement_key: "urls",
            },
        ),
        "local_path_override" => fixed_override_dependency(
            text,
            FixedOverrideRequest {
                directive,
                call_start,
                close,
                args: &args,
                source: "path",
                requirement_key: "path",
            },
        ),
        _ => None,
    }
}

fn bazel_dep_dependency(
    text: &str,
    call_start: usize,
    close: usize,
    args: &BazelArguments,
) -> ParsedBazelDependency {
    let name = args.get("name")?;
    let version = args.get("version")?;
    let registry = args.get("registry");
    let repo_name = args.get("repo_name");
    Some(Dependency {
        name: name.value.as_str().to_owned(),
        requirement: version.value.as_str().to_owned(),
        ecosystem: Bazel,
        group: "bazel_dep".to_owned(),
        hosted_url: registry.map(|value| value.value.as_str().to_owned()),
        hosted_name: repo_name.map(|value| value.value.as_str().to_owned()),
        range: offset_range(text, call_start, close + 1),
        requirement_range: offset_range(text, version.value_start, version.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn version_override_dependency(
    text: &str,
    directive: &str,
    call_start: usize,
    close: usize,
    args: &BazelArguments,
) -> ParsedBazelDependency {
    let name = args.get("module_name").or_else(|| args.get("name"))?;
    let version = args.get("version")?;
    let registry = args.get("registry");
    Some(Dependency {
        name: name.value.as_str().to_owned(),
        requirement: version.value.as_str().to_owned(),
        ecosystem: Bazel,
        group: directive.to_owned(),
        hosted_url: registry.map(|value| value.value.as_str().to_owned()),
        hosted_name: None,
        range: offset_range(text, call_start, close + 1),
        requirement_range: offset_range(text, version.value_start, version.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn fixed_override_dependency(
    text: &str,
    request: FixedOverrideRequest<'_>,
) -> ParsedBazelDependency {
    let name = request
        .args
        .get("module_name")
        .or_else(|| request.args.get("name"))?;
    let requirement = request
        .args
        .get(request.requirement_key)
        .or_else(|| request.args.get("remote"))
        .or_else(|| request.args.get("url"))
        .unwrap_or(name);
    Some(Dependency {
        name: name.value.as_str().to_owned(),
        requirement: requirement.value.as_str().to_owned(),
        ecosystem: Bazel,
        group: request.directive.to_owned(),
        hosted_url: Some(request.source.to_owned()),
        hosted_name: None,
        range: offset_range(text, request.call_start, request.close + 1),
        requirement_range: offset_range(text, requirement.value_start, requirement.value_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

#[derive(Clone)]
struct ArgumentValue {
    value: String,
    value_start: usize,
    value_end: usize,
}

fn parse_kwargs(text: &str, start: usize, end: usize) -> BazelArguments {
    let mut args: BazelArguments = crate::default();
    let mut offset = start;
    while offset < end {
        let Some((key, equals)) = next_keyword(text, offset, end) else {
            break;
        };
        let value_start = skip_ws(text, equals + 1);
        if value_start >= end {
            break;
        }
        if text.as_bytes().get(value_start) == Some(&b'"') {
            if let Some(value) = quoted_value(text, value_start) {
                offset = value.outer_end;
                args.insert(key, value.into_argument());
                continue;
            }
        } else if text.as_bytes().get(value_start) == Some(&b'[') {
            if let Some(value) = first_quoted_list_value(text, value_start) {
                offset = value.outer_end;
                args.insert(key, value.into_argument());
                continue;
            }
        }
        offset = value_start + 1;
    }
    args
}

fn next_keyword(text: &str, mut offset: usize, end: usize) -> Option<(String, usize)> {
    while offset < end {
        let byte = *text.as_bytes().get(offset)?;
        if is_identifier_start(byte) {
            let key_start = offset;
            offset += 1;
            while offset < end
                && text
                    .as_bytes()
                    .get(offset)
                    .is_some_and(|byte| is_identifier(*byte))
            {
                offset += 1;
            }
            let key = text.get(key_start..offset)?.to_owned();
            let equals = skip_ws(text, offset);
            if equals < end && text.as_bytes().get(equals) == Some(&b'=') {
                return Some((key, equals));
            }
        }
        offset += 1;
    }
    None
}

struct ParsedValue {
    value: String,
    value_start: usize,
    value_end: usize,
    outer_end: usize,
}

impl ParsedValue {
    fn into_argument(self) -> ArgumentValue {
        ArgumentValue {
            value: self.value,
            value_start: self.value_start,
            value_end: self.value_end,
        }
    }
}

fn quoted_value(text: &str, quote: usize) -> Option<ParsedValue> {
    let mut offset = quote + 1;
    let mut escaped = false;
    while offset < text.len() {
        let byte = *text.as_bytes().get(offset)?;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            let value = text.get(quote + 1..offset)?.to_owned();
            return Some(ParsedValue {
                value,
                value_start: quote + 1,
                value_end: offset,
                outer_end: offset + 1,
            });
        }
        offset += 1;
    }
    None
}

fn first_quoted_list_value(text: &str, bracket: usize) -> Option<ParsedValue> {
    let end = matching_delimiter(text, bracket, b'[', b']')?;
    let mut offset = bracket + 1;
    while offset < end {
        if text.as_bytes().get(offset) == Some(&b'"') {
            let mut value = quoted_value(text, offset)?;
            value.outer_end = end + 1;
            return Some(value);
        }
        offset += 1;
    }
    None
}

fn is_identifier_boundary(text: &str, start: usize, len: usize) -> bool {
    let before = start
        .checked_sub(1)
        .and_then(|index| text.as_bytes().get(index))
        .is_none_or(|byte| !is_identifier(*byte));
    let after = text
        .as_bytes()
        .get(start + len)
        .is_none_or(|byte| !is_identifier(*byte));
    before && after
}

fn skip_ws(text: &str, mut offset: usize) -> usize {
    while text
        .as_bytes()
        .get(offset)
        .is_some_and(|value| value.is_ascii_whitespace())
    {
        offset += 1;
    }
    offset
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}

fn is_identifier(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
