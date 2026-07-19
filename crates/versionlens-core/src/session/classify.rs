use versionlens_model::{DocumentInput, ManifestKind};
use versionlens_parsers::classify_document;

use crate::VersionLensSession;
use crate::config::FilePatternConfig;
use versionlens_model::ManifestKind::{
    DockerComposeYaml, Dockerfile, DubJson, DubSdl, PythonPipfile, PythonPyprojectToml,
    PythonRequirementsTxt, Unknown,
};

impl VersionLensSession {
    pub(crate) fn classify_document(&self, input: &DocumentInput) -> ManifestKind {
        let kind = classify_document(input);
        if kind != Unknown {
            return kind;
        }

        match configured_file_pattern_kind(&self.config.providers.file_patterns, input) {
            Some(kind) => kind,
            None => Unknown,
        }
    }
}

fn configured_file_pattern_kind(
    patterns: &[FilePatternConfig],
    input: &DocumentInput,
) -> Option<ManifestKind> {
    let path = file_path_from_uri(&input.uri)?;
    let relative_path = workspace_relative_path(path, input.workspace_root.as_deref());
    patterns
        .iter()
        .find(|config| {
            configured_pattern_matches(&config.pattern, path)
                || relative_path
                    .is_some_and(|candidate| configured_pattern_matches(&config.pattern, candidate))
        })
        .map(|config| configured_manifest_kind(config.manifest_kind, path))
}

fn file_path_from_uri(uri: &str) -> Option<&str> {
    uri.strip_prefix("file://")
}

fn workspace_relative_path<'a>(path: &'a str, workspace_root: Option<&str>) -> Option<&'a str> {
    let root = workspace_root?.trim_end_matches('/');
    let relative = path.strip_prefix(root)?.strip_prefix('/')?;
    (!relative.is_empty()).then_some(relative)
}

fn configured_pattern_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return false;
    }

    if let Some(suffix_pattern) = pattern.strip_prefix("**/") {
        return path_suffix_matches(suffix_pattern, path);
    }

    let target = if pattern.contains('/') {
        path.trim_start_matches('/')
    } else {
        file_name(path)
    };
    glob_matches(pattern, target)
}

fn path_suffix_matches(pattern: &str, path: &str) -> bool {
    let path = path.trim_start_matches('/');
    if glob_matches(pattern, path) {
        return true;
    }

    path.match_indices('/')
        .any(|(index, _)| glob_matches(pattern, &path[index + 1..]))
}

fn file_name(path: &str) -> &str {
    match path.rsplit('/').next() {
        Some(name) => name,
        None => path,
    }
}

fn configured_manifest_kind(kind: ManifestKind, path: &str) -> ManifestKind {
    match kind {
        DockerComposeYaml if !has_extension(path, ["yaml", "yml"]) => Dockerfile,
        PythonRequirementsTxt if has_extension(path, ["toml"]) => PythonPyprojectToml,
        PythonRequirementsTxt if file_name(path).eq_ignore_ascii_case("Pipfile") => PythonPipfile,
        DubJson if has_extension(path, ["sdl"]) => DubSdl,
        _ => kind,
    }
}

fn has_extension<const N: usize>(path: &str, extensions: [&str; N]) -> bool {
    let name = file_name(path);
    let Some((_, extension)) = name.rsplit_once('.') else {
        return false;
    };
    extensions
        .iter()
        .any(|candidate| extension.eq_ignore_ascii_case(candidate))
}

fn glob_matches(pattern: &str, text: &str) -> bool {
    if let Some((prefix, alternatives, suffix)) = extglob_alternative_parts(pattern) {
        return alternatives.split('|').any(|alternative| {
            let mut expanded: String = crate::default();
            expanded.push_str(prefix);
            expanded.push_str(alternative);
            expanded.push_str(suffix);
            glob_matches(&expanded, text)
        });
    }

    if let Some((prefix, alternatives, suffix)) = brace_parts(pattern) {
        return alternatives.split(',').any(|alternative| {
            let mut expanded: String = crate::default();
            expanded.push_str(prefix);
            expanded.push_str(alternative);
            expanded.push_str(suffix);
            glob_matches(&expanded, text)
        });
    }

    glob_matches_bytes(pattern.as_bytes(), text.as_bytes())
}

fn extglob_alternative_parts(pattern: &str) -> Option<(&str, &str, &str)> {
    let open = pattern.find("@(")?;
    let alternatives_start = open + 2;
    let close = matching_close_paren(pattern, alternatives_start)?;
    Some((
        &pattern[..open],
        &pattern[alternatives_start..close],
        &pattern[close + 1..],
    ))
}

fn matching_close_paren(pattern: &str, alternatives_start: usize) -> Option<usize> {
    let mut depth = 0_u32;
    for (offset, value) in pattern
        .char_indices()
        .skip_while(|(offset, _)| *offset < alternatives_start)
    {
        match value {
            '(' => depth = depth.saturating_add(1),
            ')' if depth == 0 => return Some(offset),
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    None
}

fn brace_parts(pattern: &str) -> Option<(&str, &str, &str)> {
    let open = pattern.find('{')?;
    let rest = &pattern[open + 1..];
    let close = rest.find('}')?;
    Some((&pattern[..open], &rest[..close], &rest[close + 1..]))
}

fn glob_matches_bytes(pattern: &[u8], text: &[u8]) -> bool {
    let Some((&head, rest)) = pattern.split_first() else {
        return text.is_empty();
    };

    match head {
        b'*' if rest.first() == Some(&b'*') => double_star_matches(&rest[1..], text),
        b'*' => star_matches(rest, text),
        b'?' => text
            .split_first()
            .is_some_and(|(&value, tail)| value != b'/' && glob_matches_bytes(rest, tail)),
        b'[' => class_matches(rest, text),
        value => text
            .split_first()
            .is_some_and(|(&candidate, tail)| candidate == value && glob_matches_bytes(rest, tail)),
    }
}

fn class_matches(pattern: &[u8], text: &[u8]) -> bool {
    let Some((&candidate, tail)) = text.split_first() else {
        return false;
    };
    let Some((class, rest)) = split_class(pattern) else {
        return candidate == b'[' && glob_matches_bytes(pattern, tail);
    };
    if candidate == b'/' {
        return false;
    }

    let (negated, members) = if let Some(members) = class.strip_prefix(b"!") {
        (true, members)
    } else {
        (false, class)
    };
    let matched = class_member_matches(members, candidate);
    if matched == negated {
        return false;
    }

    glob_matches_bytes(rest, tail)
}

fn split_class(pattern: &[u8]) -> Option<(&[u8], &[u8])> {
    let close = pattern.iter().position(|value| *value == b']')?;
    Some((&pattern[..close], &pattern[close + 1..]))
}

fn class_member_matches(members: &[u8], candidate: u8) -> bool {
    let mut index = 0;
    while index < members.len() {
        let value = members[index];
        if index + 2 < members.len() && members[index + 1] == b'-' {
            if range_contains(value, members[index + 2], candidate) {
                return true;
            }
            index += 3;
            continue;
        }
        if value == candidate {
            return true;
        }
        index += 1;
    }

    false
}

fn range_contains(start: u8, end: u8, candidate: u8) -> bool {
    if start <= end {
        return start <= candidate && candidate <= end;
    }

    end <= candidate && candidate <= start
}

fn double_star_matches(pattern: &[u8], text: &[u8]) -> bool {
    if glob_matches_bytes(pattern, text) {
        return true;
    }
    if let Some(pattern_without_slash) = pattern.strip_prefix(b"/")
        && glob_matches_bytes(pattern_without_slash, text)
    {
        return true;
    }

    for index in 0..=text.len() {
        if glob_matches_bytes(pattern, &text[index..]) {
            return true;
        }
    }

    false
}

fn star_matches(pattern: &[u8], text: &[u8]) -> bool {
    if glob_matches_bytes(pattern, text) {
        return true;
    }

    for index in 0..text.len() {
        if text[index] == b'/' {
            return false;
        }
        if glob_matches_bytes(pattern, &text[index + 1..]) {
            return true;
        }
    }

    false
}
