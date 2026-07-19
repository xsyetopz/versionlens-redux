use self::CljTokenKind::{
    LBrace as CljLBrace, LBracket as CljLBracket, RBrace as CljRBrace, RBracket as CljRBracket,
    String as CljString,
};
use crate::maven_xml::MavenNamedRepository;
use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

type LeiningenDependencies = Vec<Dependency>;

pub(crate) fn parse_leiningen_project_clj(text: &str) -> Vec<Dependency> {
    let tokens = tokenize_clj(text);
    let mut dependencies = vec![];

    if let Some(version) = project_version_dependency(text, &tokens) {
        dependencies.push(version);
    }
    collect_dependency_vectors(text, &tokens, &mut dependencies);

    dependencies
}

pub fn parse_leiningen_maven_repositories(text: &str) -> Vec<MavenNamedRepository> {
    let tokens = tokenize_clj(text);
    let Some(repositories_key) = tokens
        .iter()
        .position(|token| token.text == ":repositories")
    else {
        return vec![];
    };
    let Some(CljLBracket) = tokens.get(repositories_key + 1).map(|token| token.kind) else {
        return vec![];
    };
    let Some(repositories_end) =
        matching_delimited(&tokens, repositories_key + 1, CljLBracket, CljRBracket)
    else {
        return vec![];
    };

    let mut repositories = vec![];
    let mut index = repositories_key + 2;
    while index < repositories_end {
        if tokens[index].kind != CljLBracket {
            index += 1;
            continue;
        }
        let Some(repository_end) = matching_delimited(&tokens, index, CljLBracket, CljRBracket)
        else {
            index += 1;
            continue;
        };
        if let Some(repository) = leiningen_repository_entry(&tokens, index, repository_end) {
            repositories.push(repository);
        }
        index = repository_end + 1;
    }

    repositories
}

fn leiningen_repository_entry(
    tokens: &CljTokens<'_>,
    start: usize,
    end: usize,
) -> Option<MavenNamedRepository> {
    let id = tokens.get(start + 1)?.text;
    let url = match tokens.get(start + 2)? {
        token if token.kind == CljString => token.text,
        token if token.kind == CljLBrace => {
            let map_end = matching_delimited(tokens, start + 2, CljLBrace, CljRBrace)?;
            if map_end > end {
                return None;
            }
            leiningen_repository_url(tokens, start + 3, map_end)?
        }
        _ => return None,
    };

    Some(MavenNamedRepository {
        id: id.trim_start_matches(':').to_owned(),
        url: url.to_owned(),
    })
}

fn leiningen_repository_url<'a>(
    tokens: &'a CljTokens<'a>,
    start: usize,
    end: usize,
) -> Option<&'a str> {
    let url_key = (start..end).find(|index| tokens[*index].text == ":url")?;
    tokens.get(url_key + 1).map(|token| token.text)
}

fn project_version_dependency(text: &str, tokens: &CljTokens<'_>) -> Option<Dependency> {
    let defproject = tokens.iter().position(|token| token.text == "defproject")?;
    let name = tokens.get(defproject + 1)?;
    let version = tokens.get(defproject + 2)?;
    if version.kind != CljString {
        return None;
    }

    Some(Dependency {
        name: name.text.to_owned(),
        requirement: version.text.to_owned(),
        ecosystem: Maven,
        group: "version".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, name.start, version.end),
        requirement_range: offset_range(text, version.content_start, version.content_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn collect_dependency_vectors(
    text: &str,
    tokens: &CljTokens<'_>,
    dependencies: &mut LeiningenDependencies,
) {
    for index in 0..tokens.len() {
        let Some(group) = dependency_group(tokens, index) else {
            continue;
        };
        let Some(CljLBracket) = tokens.get(index + 1).map(|token| token.kind) else {
            continue;
        };
        let Some(end) = matching_delimited(tokens, index + 1, CljLBracket, CljRBracket) else {
            continue;
        };
        collect_dependency_entries(
            LeiningenDependencyEntries {
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

fn dependency_group(tokens: &CljTokens<'_>, index: usize) -> Option<String> {
    if tokens.get(index)?.text == ":dependencies"
        && let Some(profile) = enclosing_profile(tokens, index)
    {
        return Some(format!("profiles.{profile}.dependencies"));
    }

    match tokens.get(index)?.text {
        ":dependencies" | ":managed-dependencies" | ":plugins" => {
            Some(tokens[index].text.trim_start_matches(':').to_owned())
        }
        _ => None,
    }
}

fn enclosing_profile<'a>(tokens: &'a CljTokens<'a>, index: usize) -> Option<&'a str> {
    let profiles_index = tokens[..index]
        .iter()
        .enumerate()
        .rfind(|(_, token)| token.text == ":profiles")
        .map(|(index, _)| index)?;
    let mut profile = None;

    for cursor in profiles_index + 1..index {
        if tokens[cursor].kind != CljLBrace || cursor == 0 {
            continue;
        }
        if !tokens[cursor - 1].text.starts_with(':') || tokens[cursor - 1].text == ":dependencies" {
            continue;
        }
        if matching_delimited(tokens, cursor, CljLBrace, CljRBrace).is_some_and(|end| end >= index)
        {
            profile = tokens[cursor - 1].text.strip_prefix(':');
        }
    }

    profile
}

struct LeiningenDependencyEntries<'a, 'tokens> {
    text: &'a str,
    tokens: &'a CljTokens<'tokens>,
    group: &'a str,
    dependencies: &'a mut LeiningenDependencies,
}

fn collect_dependency_entries(
    context: LeiningenDependencyEntries<'_, '_>,
    mut index: usize,
    end: usize,
) {
    while index < end {
        if context.tokens[index].kind != CljLBracket {
            index += 1;
            continue;
        }
        let Some(entry_end) = matching_delimited(context.tokens, index, CljLBracket, CljRBracket)
        else {
            index += 1;
            continue;
        };
        if let Some(dependency) = dependency_entry(
            context.text,
            context.tokens,
            index,
            entry_end,
            context.group,
        ) {
            context.dependencies.push(dependency);
        }
        index = entry_end + 1;
    }
}

fn dependency_entry(
    text: &str,
    tokens: &CljTokens<'_>,
    start: usize,
    end: usize,
    group: &str,
) -> Option<Dependency> {
    let name = tokens.get(start + 1)?;
    let version = tokens.get(start + 2)?;
    if version.kind != CljString {
        return None;
    }

    Some(Dependency {
        name: leiningen_maven_name(name.text),
        requirement: version.text.to_owned(),
        ecosystem: Maven,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: offset_range(text, tokens[start].start, tokens[end].end),
        requirement_range: offset_range(text, version.content_start, version.content_end),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    })
}

fn leiningen_maven_name(raw: &str) -> String {
    let raw = raw.trim_matches('"');
    if let Some((group, artifact)) = raw.split_once('/') {
        return format!("{group}:{artifact}");
    }
    format!("{raw}:{raw}")
}

fn matching_delimited(
    tokens: &CljTokens<'_>,
    start: usize,
    open: CljTokenKind,
    close: CljTokenKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind == open {
            depth += 1;
        } else if token.kind == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

use CljTokenKind::{LParen as CljLParen, RParen as CljRParen, Symbol as CljSymbol};

#[derive(Clone, Copy, PartialEq, Eq)]
enum CljTokenKind {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    String,
    Symbol,
}

#[derive(Clone, Copy)]
struct CljToken<'a> {
    kind: CljTokenKind,
    text: &'a str,
    start: usize,
    end: usize,
    content_start: usize,
    content_end: usize,
}

type CljTokens<'a> = [CljToken<'a>];

fn tokenize_clj(text: &str) -> Vec<CljToken<'_>> {
    let mut tokens = vec![];
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        match bytes[index] {
            b'(' => {
                tokens.push(single_char_token(CljLParen, text, index));
                index += 1;
            }
            b')' => {
                tokens.push(single_char_token(CljRParen, text, index));
                index += 1;
            }
            b'[' => {
                tokens.push(single_char_token(CljLBracket, text, index));
                index += 1;
            }
            b']' => {
                tokens.push(single_char_token(CljRBracket, text, index));
                index += 1;
            }
            b'{' => {
                tokens.push(single_char_token(CljLBrace, text, index));
                index += 1;
            }
            b'}' => {
                tokens.push(single_char_token(CljRBrace, text, index));
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
                    tokens.push(CljToken {
                        kind: CljString,
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
                while index < bytes.len() && !is_clj_delimiter(bytes[index]) {
                    index += 1;
                }
                if let Some(value) = text.get(start..index) {
                    tokens.push(CljToken {
                        kind: CljSymbol,
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

fn single_char_token(kind: CljTokenKind, text: &str, index: usize) -> CljToken<'_> {
    CljToken {
        kind,
        text: text.get(index..index + 1).unwrap_or_default(),
        start: index,
        end: index + 1,
        content_start: index,
        content_end: index + 1,
    }
}

fn is_clj_delimiter(byte: u8) -> bool {
    byte.is_ascii_whitespace()
        || matches!(
            byte,
            b'(' | b')' | b'[' | b']' | b'{' | b'}' | b'"' | b',' | b';'
        )
}
