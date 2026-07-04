use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_vscode_model::{Position, Range};

use crate::model::SuggestionStatus;

use super::{resolve_with_latest, unresolved};

#[test]
fn unresolved_marks_dependencies_unresolved() {
    let suggestions = unresolved(vec![dependency("serde", "1.0.0")]);

    assert_eq!(suggestions[0].status, SuggestionStatus::Unresolved);
    assert_eq!(suggestions[0].latest, None);
}

#[test]
fn latest_marks_update_status() {
    let suggestions = resolve_with_latest(
        vec![dependency("serde", "1.0.0"), dependency("tokio", "2.0.0")],
        "1.5.0",
    );

    assert_eq!(suggestions[0].status, SuggestionStatus::UpdateAvailable);
    assert_eq!(suggestions[1].status, SuggestionStatus::Current);
}

#[test]
fn latest_satisfying_range_is_current() {
    let suggestions = resolve_with_latest(vec![dependency("serde", "^1.0.0")], "1.5.0");

    assert_eq!(suggestions[0].status, SuggestionStatus::SatisfiesLatest);
}

fn dependency(name: &str, requirement: &str) -> Dependency {
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ecosystem::Cargo,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
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
