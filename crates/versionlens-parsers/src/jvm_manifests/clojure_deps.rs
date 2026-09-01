use crate::edn::{TokenKind, Tokens, field_value, matching, tokenize, visit_delimited_groups};
use crate::maven_xml::MavenNamedRepository;
use crate::positions::offset_range;
use TokenKind::{
    Keyword as EdnKeyword, LBrace as EdnLBrace, RBrace as EdnRBrace, String as EdnString,
    Symbol as EdnSymbol,
};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Maven;

type ClojureDependencies = Vec<Dependency>;

pub(crate) fn parse_clojure_deps_edn(text: &str) -> Vec<Dependency> {
    let tokens = tokenize(text);
    let mut dependencies = vec![];
    collect_deps_maps(text, &tokens, &mut dependencies);
    dependencies
}

pub fn parse_clojure_maven_repositories(text: &str) -> Vec<MavenNamedRepository> {
    let tokens = tokenize(text);
    let Some(repos_key) = tokens.iter().position(|token| token.text == ":mvn/repos") else {
        return vec![];
    };
    let Some(EdnLBrace) = tokens.get(repos_key + 1).map(|token| token.kind) else {
        return vec![];
    };
    let Some(repos_end) = matching(&tokens, repos_key + 1, EdnLBrace, EdnRBrace) else {
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
        let Some(repository_end) = matching(&tokens, index + 1, EdnLBrace, EdnRBrace) else {
            index += 1;
            continue;
        };
        if let Some(url) = field_value(&tokens, index + 2, repository_end, ":url") {
            repositories.push(MavenNamedRepository {
                id: repository_id(id_token.text),
                url: url.to_owned(),
            });
        }
        index = repository_end + 1;
    }

    repositories
}

fn repository_id(text: &str) -> String {
    text.strip_prefix(':').unwrap_or(text).to_owned()
}

fn collect_deps_maps(text: &str, tokens: &Tokens<'_>, dependencies: &mut ClojureDependencies) {
    visit_delimited_groups(
        tokens,
        EdnLBrace,
        EdnRBrace,
        deps_group,
        |group, start, end| {
            collect_dependency_entries(
                ClojureDependencyEntries {
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

fn deps_group(tokens: &Tokens<'_>, index: usize) -> Option<String> {
    match tokens.get(index)?.text {
        ":deps" => Some("deps".to_owned()),
        ":extra-deps" | ":override-deps" | ":default-deps" | ":replace-deps" => {
            let alias = enclosing_alias(tokens, index)?;
            Some(format!("aliases.{alias}.{}", &tokens[index].text[1..]))
        }
        _ => None,
    }
}

fn enclosing_alias<'a>(tokens: &'a Tokens<'a>, index: usize) -> Option<&'a str> {
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
        if matching(tokens, cursor, EdnLBrace, EdnRBrace).is_some_and(|end| end >= index) {
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
    tokens: &'a Tokens<'tokens>,
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
        let Some(coord_end) = matching(context.tokens, index + 1, EdnLBrace, EdnRBrace) else {
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
    tokens: &Tokens<'_>,
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
        canonical_reference: None,
    })
}

fn clojure_requirement<'a>(
    tokens: &'a Tokens<'a>,
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
