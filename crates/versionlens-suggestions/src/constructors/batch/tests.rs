use crate::suggestion::SuggestionStatus::{
    Current as StatusCurrent, SatisfiesLatest as StatusSatisfiesLatest,
    Unresolved as StatusUnresolved, UpdateAvailable as StatusUpdateAvailable,
};
use versionlens_model::Dependency;
use versionlens_model::{Position, Range};

use super::{resolve_with_latest, unresolved};
use versionlens_model::Ecosystem::Cargo;

#[test]
fn unresolved_marks_dependencies_unresolved() {
    let suggestions = unresolved(vec![dependency("serde", "1.0.0")]);

    assert_eq!(suggestions[0].status, StatusUnresolved);
    assert_eq!(suggestions[0].latest, None);
}

#[test]
fn latest_marks_update_status() {
    let suggestions = resolve_with_latest(
        vec![dependency("serde", "1.0.0"), dependency("tokio", "2.0.0")],
        "1.5.0",
    );

    assert_eq!(suggestions[0].status, StatusUpdateAvailable);
    assert_eq!(suggestions[1].status, StatusCurrent);
}

#[test]
fn latest_satisfying_range_is_current() {
    let suggestions = resolve_with_latest(vec![dependency("serde", "^1.0.0")], "1.5.0");

    assert_eq!(suggestions[0].status, StatusSatisfiesLatest);
}

fn dependency(name: &str, requirement: &str) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Cargo,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn empty_range() -> Range {
    Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}
