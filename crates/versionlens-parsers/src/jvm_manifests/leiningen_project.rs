use crate::edn::{TokenKind, Tokens, field_value, matching, tokenize, visit_delimited_groups};
use crate::maven_xml::MavenNamedRepository;
use crate::positions::offset_range;
use TokenKind::{
    LBrace as CljLBrace, LBracket as CljLBracket, RBrace as CljRBrace, RBracket as CljRBracket,
    String as CljString,
};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

type LeiningenDependencies = Vec<Dependency>;

pub(crate) fn parse_leiningen_project_clj(text: &str) -> Vec<Dependency> {
    let tokens = tokenize(text);
    let mut dependencies = vec![];

    if let Some(version) = project_version_dependency(text, &tokens) {
        dependencies.push(version);
    }
    collect_dependency_vectors(text, &tokens, &mut dependencies);

    dependencies
}

pub fn parse_leiningen_maven_repositories(text: &str) -> Vec<MavenNamedRepository> {
    let tokens = tokenize(text);
    let Some(repositories_key) = tokens
        .iter()
        .position(|token| token.text == ":repositories")
    else {
        return vec![];
    };
    let Some(CljLBracket) = tokens.get(repositories_key + 1).map(|token| token.kind) else {
        return vec![];
    };
    let Some(repositories_end) = matching(&tokens, repositories_key + 1, CljLBracket, CljRBracket)
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
        let Some(repository_end) = matching(&tokens, index, CljLBracket, CljRBracket) else {
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
    tokens: &Tokens<'_>,
    start: usize,
    end: usize,
) -> Option<MavenNamedRepository> {
    let id = tokens.get(start + 1)?.text;
    let url = match tokens.get(start + 2)? {
        token if token.kind == CljString => token.text,
        token if token.kind == CljLBrace => {
            let map_end = matching(tokens, start + 2, CljLBrace, CljRBrace)?;
            if map_end > end {
                return None;
            }
            field_value(tokens, start + 3, map_end, ":url")?
        }
        _ => return None,
    };

    Some(MavenNamedRepository {
        id: id.trim_start_matches(':').to_owned(),
        url: url.to_owned(),
    })
}

fn project_version_dependency(text: &str, tokens: &Tokens<'_>) -> Option<Dependency> {
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
    tokens: &Tokens<'_>,
    dependencies: &mut LeiningenDependencies,
) {
    visit_delimited_groups(
        tokens,
        CljLBracket,
        CljRBracket,
        dependency_group,
        |group, start, end| {
            collect_dependency_entries(
                LeiningenDependencyEntries {
                    text,
                    tokens,
                    group: &group,
                    dependencies,
                },
                start,
                end,
            );
        },
    );
}

fn dependency_group(tokens: &Tokens<'_>, index: usize) -> Option<String> {
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

fn enclosing_profile<'a>(tokens: &'a Tokens<'a>, index: usize) -> Option<&'a str> {
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
        if matching(tokens, cursor, CljLBrace, CljRBrace).is_some_and(|end| end >= index) {
            profile = tokens[cursor - 1].text.strip_prefix(':');
        }
    }

    profile
}

struct LeiningenDependencyEntries<'a, 'tokens> {
    text: &'a str,
    tokens: &'a Tokens<'tokens>,
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
        let Some(entry_end) = matching(context.tokens, index, CljLBracket, CljRBracket) else {
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
    tokens: &Tokens<'_>,
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
