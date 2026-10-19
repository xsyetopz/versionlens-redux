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
