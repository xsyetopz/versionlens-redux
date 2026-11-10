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

/// The dependencies a selector names. A selector made before an edit can
/// name a range that moved; it then falls back to its name, but only when
/// one dependency has that name, so it never picks another entry.
pub(crate) fn select_dependencies(
    dependencies: Vec<Dependency>,
    selector: &str,
) -> Vec<Dependency> {
    let name = selector_name(selector);
    let named = dependencies
        .iter()
        .filter(|dependency| dependency.name == name)
        .count();
    let exact = dependencies
        .iter()
        .any(|dependency| matches_dependency(dependency, selector));
    dependencies
        .into_iter()
        .filter(|dependency| {
            if exact {
                matches_dependency(dependency, selector)
            } else {
                named == 1 && dependency.name == name
            }
        })
        .collect()
}

/// The dependency name of a selector.
pub(crate) fn selector_name(selector: &str) -> &str {
    selector
        .split_once(SEPARATOR)
        .map_or(selector, |(name, _)| name)
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

#[cfg(test)]
mod tests {
use versionlens_model::{Dependency, Ecosystem, Position, Range};

use super::{dependency_selector, select_dependencies};

fn dependency(name: &str, line: u32) -> Dependency {
    let range = Range {
        start: Position {
            line,
            character: 10,
        },
        end: Position {
            line,
            character: 15,
        },
    };
    Dependency {
        name: name.to_owned(),
        requirement: "1.0.0".to_owned(),
        ecosystem: Ecosystem::Npm,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

fn lines(selected: &[Dependency]) -> Vec<u32> {
    selected
        .iter()
        .map(|dependency| dependency.requirement_range.start.line)
        .collect()
}

#[test]
fn a_moved_selector_falls_back_to_a_unique_name() {
    let selector = dependency_selector(&dependency("beta", 1));
    let moved = vec![dependency("alpha", 1), dependency("beta", 2)];

    assert_eq!(lines(&select_dependencies(moved, &selector)), [2]);
}

#[test]
fn a_moved_selector_with_a_shared_name_selects_nothing() {
    let selector = dependency_selector(&dependency("beta", 1));
    let moved = vec![dependency("beta", 2), dependency("beta", 3)];

    assert!(select_dependencies(moved, &selector).is_empty());
}

#[test]
fn an_exact_selector_picks_its_entry_among_shared_names() {
    let selector = dependency_selector(&dependency("beta", 3));
    let current = vec![dependency("beta", 2), dependency("beta", 3)];

    assert_eq!(lines(&select_dependencies(current, &selector)), [3]);
}
}
