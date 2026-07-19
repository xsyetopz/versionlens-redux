use self::EdnTokenKind::{
    Keyword as EdnKeyword, LBrace as EdnLBrace, LBracket as EdnLBracket, RBrace as EdnRBrace,
    RBracket as EdnRBracket, String as EdnString, Symbol as EdnSymbol,
};
use crate::maven_xml::MavenNamedRepository;
use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

type ClojureDependencies = Vec<Dependency>;

pub(crate) fn parse_clojure_deps_edn(text: &str) -> Vec<Dependency> {
    let tokens = tokenize_edn(text);
    let mut dependencies = vec![];
    collect_deps_maps(text, &tokens, &mut dependencies);
    dependencies
}

pub fn parse_clojure_maven_repositories(text: &str) -> Vec<MavenNamedRepository> {
    let tokens = tokenize_edn(text);
    let Some(repos_key) = tokens.iter().position(|token| token.text == ":mvn/repos") else {
        return vec![];
    };
    let Some(EdnLBrace) = tokens.get(repos_key + 1).map(|token| token.kind) else {
        return vec![];
    };
    let Some(repos_end) = matching_brace(&tokens, repos_key + 1) else {
        return vec![];
    };

    let mut repositories = vec![];
    let mut index = repos_key + 2;
    while index < repos_end {
        let Some(id_token) = tokens.get(index) else {
            break;
        };
        if !matches!(id_token.kind, EdnString | EdnSymbol | EdnKeyword) {
            index += 1;
            continue;
        }
        let Some(EdnLBrace) = tokens.get(index + 1).map(|token| token.kind) else {
            index += 1;
            continue;
        };
        let Some(repository_end) = matching_brace(&tokens, index + 1) else {
            index += 1;
            continue;
        };
        if let Some(url) = clojure_maven_repository_url(&tokens, index + 2, repository_end) {
            repositories.push(MavenNamedRepository {
                id: repository_id(id_token.text),
                url: url.to_owned(),
            });
        }
        index = repository_end + 1;
    }

    repositories
}

fn clojure_maven_repository_url<'a>(
    tokens: &'a EdnTokens<'a>,
    start: usize,
    end: usize,
) -> Option<&'a str> {
    let url_key = (start..end).find(|index| tokens[*index].text == ":url")?;
    tokens.get(url_key + 1).map(|token| token.text)
}

fn repository_id(text: &str) -> String {
    text.strip_prefix(':').unwrap_or(text).to_owned()
}

fn collect_deps_maps(text: &str, tokens: &EdnTokens<'_>, dependencies: &mut ClojureDependencies) {
    for index in 0..tokens.len() {
        let Some(group) = deps_group(tokens, index) else {
            continue;
        };
        let Some(EdnLBrace) = tokens.get(index + 1).map(|token| token.kind) else {
            continue;
        };
        let Some(end) = matching_brace(tokens, index + 1) else {
            continue;
        };
        collect_dependency_entries(
            ClojureDependencyEntries {
                text,
                tokens,
                group: &group,
                dependencies,
            },
            index + 2,
            end,
        );
    }
}

fn deps_group(tokens: &EdnTokens<'_>, index: usize) -> Option<String> {
    match tokens.get(index)?.text {
        ":deps" => Some("deps".to_owned()),
        ":extra-deps" | ":override-deps" | ":default-deps" | ":replace-deps" => {
            let alias = enclosing_alias(tokens, index)?;
            Some(format!("aliases.{alias}.{}", &tokens[index].text[1..]))
        }
        _ => None,
    }
}

fn enclosing_alias<'a>(tokens: &'a EdnTokens<'a>, index: usize) -> Option<&'a str> {
    let aliases_index = tokens[..index]
        .iter()
        .enumerate()
        .rfind(|(_, token)| token.text == ":aliases")
        .map(|(index, _)| index)?;
    let mut alias = None;

    for cursor in aliases_index + 1..index {
        if tokens[cursor].kind != EdnLBrace || cursor == 0 {
            continue;
        }
        if !tokens[cursor - 1].text.starts_with(':') || is_deps_group_key(tokens[cursor - 1].text) {
            continue;
        }
        if matching_brace(tokens, cursor).is_some_and(|end| end >= index) {
            alias = tokens[cursor - 1].text.strip_prefix(':');
        }
    }

    alias
}

fn is_deps_group_key(text: &str) -> bool {
    matches!(
        text,
        ":deps" | ":extra-deps" | ":override-deps" | ":default-deps" | ":replace-deps"
    )
}

struct ClojureDependencyEntries<'a, 'tokens> {
    text: &'a str,
    tokens: &'a EdnTokens<'tokens>,
    group: &'a str,
    dependencies: &'a mut ClojureDependencies,
}

fn collect_dependency_entries(
    context: ClojureDependencyEntries<'_, '_>,
    mut index: usize,
    end: usize,
) {
    while index < end {
        let Some(name_token) = context.tokens.get(index) else {
            break;
        };
        if !matches!(name_token.kind, EdnSymbol | EdnKeyword) {
            index += 1;
            continue;
        }
        let Some(EdnLBrace) = context.tokens.get(index + 1).map(|token| token.kind) else {
            index += 1;
            continue;
        };
        let Some(coord_end) = matching_brace(context.tokens, index + 1) else {
            index += 1;
            continue;
        };
        if let Some(dependency) = clojure_dependency(
            context.text,
            context.tokens,
            index,
            coord_end,
            context.group,
        ) {
            context.dependencies.push(dependency);
        }
        index = coord_end + 1;
    }
}

fn clojure_dependency(
    text: &str,
    tokens: &EdnTokens<'_>,
    name_index: usize,
    coord_end: usize,
    group: &str,
) -> Option<Dependency> {
    let raw_name = tokens[name_index]
        .text
        .strip_prefix(':')
        .unwrap_or(tokens[name_index].text);
    let name = clojure_maven_name(raw_name);
    let coord_start = name_index + 2;
    let (requirement, requirement_span, hosted_url) =
        clojure_requirement(tokens, coord_start, coord_end)?;

    Some(Dependency {
        name,
        requirement: requirement.to_owned(),
        ecosystem: Maven,
        group: group.to_owned(),
        hosted_url: hosted_url.map(|value| value.to_owned()),
        hosted_name: None,
        range: offset_range(text, tokens[name_index].start, tokens[coord_end].end),
        requirement_range: offset_range(text, requirement_span.0, requirement_span.1),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn clojure_requirement<'a>(
    tokens: &'a EdnTokens<'a>,
    start: usize,
    end: usize,
) -> Option<(&'a str, (usize, usize), Option<&'static str>)> {
    for field in [":mvn/version", ":git/tag", ":git/url", ":local/root"] {
        let Some(field_index) = (start..end).find(|index| tokens[*index].text == field) else {
            continue;
        };
        let value = tokens.get(field_index + 1)?;
        let hosted_url = match field {
            ":git/tag" | ":git/url" => Some("git"),
            ":local/root" => Some("local"),
            _ => None,
        };
        return Some((
            value.text,
            (value.content_start, value.content_end),
            hosted_url,
        ));
    }

    None
}

fn clojure_maven_name(raw: &str) -> String {
    if let Some((group, artifact)) = raw.split_once('/') {
        let artifact = artifact
            .split_once('$')
            .map_or(artifact, |(artifact, _)| artifact);
        return format!("{group}:{artifact}");
    }

    format!("{raw}:{raw}")
}

fn matching_brace(tokens: &EdnTokens<'_>, start: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        match token.kind {
            EdnLBrace => depth += 1,
            EdnRBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            EdnLBracket | EdnRBracket | EdnSymbol | EdnKeyword | EdnString => {}
        }
    }

    None
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EdnTokenKind {
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    String,
    Keyword,
    Symbol,
}

#[derive(Clone, Copy)]
struct EdnToken<'a> {
    kind: EdnTokenKind,
    text: &'a str,
    start: usize,
    end: usize,
    content_start: usize,
    content_end: usize,
}

type EdnTokens<'a> = [EdnToken<'a>];

fn tokenize_edn(text: &str) -> Vec<EdnToken<'_>> {
    let mut tokens = vec![];
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'{' => {
                tokens.push(single_char_token(EdnLBrace, text, index));
                index += 1;
            }
            b'}' => {
                tokens.push(single_char_token(EdnRBrace, text, index));
                index += 1;
            }
            b'[' => {
                tokens.push(single_char_token(EdnLBracket, text, index));
                index += 1;
            }
            b']' => {
                tokens.push(single_char_token(EdnRBracket, text, index));
                index += 1;
            }
            b'"' => {
                let start = index;
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index += 2;
                    } else if bytes[index] == b'"' {
                        break;
                    } else {
                        index += 1;
                    }
                }
                let content_end = index.min(bytes.len());
                let end = (index + 1).min(bytes.len());
                if let Some(value) = text.get(start + 1..content_end) {
                    tokens.push(EdnToken {
                        kind: EdnString,
                        text: value,
                        start,
                        end,
                        content_start: start + 1,
                        content_end,
                    });
                }
                index = end;
            }
            b';' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            byte if byte.is_ascii_whitespace() || byte == b',' => {
                index += 1;
            }
            _ => {
                let start = index;
                while index < bytes.len() && !is_edn_delimiter(bytes[index]) {
                    index += 1;
                }
                if let Some(value) = text.get(start..index) {
                    let kind = if value.starts_with(':') {
                        EdnKeyword
                    } else {
                        EdnSymbol
                    };
                    tokens.push(EdnToken {
                        kind,
                        text: value,
                        start,
                        end: index,
                        content_start: start,
                        content_end: index,
                    });
                }
            }
        }
    }

    tokens
}

fn single_char_token(kind: EdnTokenKind, text: &str, index: usize) -> EdnToken<'_> {
    EdnToken {
        kind,
        text: text.get(index..index + 1).unwrap_or_default(),
        start: index,
        end: index + 1,
        content_start: index,
        content_end: index + 1,
    }
}

fn is_edn_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'{' | b'}' | b'[' | b']' | b'"' | b',' | b';')
}
