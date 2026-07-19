use versionlens_model::Dependency;
use versionlens_model::{Position, Range};

const SEPARATOR: char = '\u{001F}';

pub(crate) fn dependency_selector(dependency: &Dependency) -> String {
    format!(
        "{}{SEPARATOR}{}:{},{}:{}",
        dependency.name,
        dependency.requirement_range.start.line,
        dependency.requirement_range.start.character,
        dependency.requirement_range.end.line,
        dependency.requirement_range.end.character,
    )
}

pub(crate) fn matches_dependency(dependency: &Dependency, selector: &str) -> bool {
    dependency.name == selector
        || parse_selector(selector).is_some_and(|selection| {
            selection.name == dependency.name
                && selection.requirement_range == dependency.requirement_range
        })
}

#[derive(Debug, PartialEq, Eq)]
struct DependencySelection {
    name: String,
    requirement_range: Range,
}

fn parse_selector(selector: &str) -> Option<DependencySelection> {
    let (name, range) = selector.split_once(SEPARATOR)?;
    let (start, end) = range.split_once(',')?;
    Some(DependencySelection {
        name: name.to_owned(),
        requirement_range: Range {
            start: parse_position(start)?,
            end: parse_position(end)?,
        },
    })
}

fn parse_position(position: &str) -> Option<Position> {
    let (line, character) = position.split_once(':')?;
    Some(Position {
        line: line.parse().ok()?,
        character: character.parse().ok()?,
    })
}
