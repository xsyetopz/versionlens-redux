use versionlens_model::Dependency;

use crate::resolve::resolve_dependency;
use crate::suggestion::Suggestion;

type Dependencies = Vec<Dependency>;
type Suggestions = Vec<Suggestion>;

pub fn unresolved(dependencies: Dependencies) -> Suggestions {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, None))
        .collect()
}

pub fn resolve_with_latest(dependencies: Dependencies, latest: &str) -> Suggestions {
    dependencies
        .into_iter()
        .map(|dependency| resolve_dependency(dependency, Some(latest.to_owned())))
        .collect()
}

#[cfg(test)]
mod tests {
use crate::suggestion::SuggestionStatus::{
    Current as StatusCurrent, SatisfiesLatest as StatusSatisfiesLatest,
    Unresolved as StatusUnresolved,
};

use super::{resolve_with_latest, unresolved};

#[test]
fn unresolved_marks_dependencies_unresolved() {
    let suggestions = unresolved(vec![crate::support::tests::test_dependency(
        "serde", "1.0.0",
    )]);

    assert_eq!(suggestions[0].status, StatusUnresolved);
    assert_eq!(suggestions[0].latest, None);
}

#[test]
fn latest_marks_update_status() {
    let suggestions = resolve_with_latest(
        vec![
            crate::support::tests::test_dependency("serde", "1.0.0"),
            crate::support::tests::test_dependency("tokio", "2.0.0"),
        ],
        "1.5.0",
    );

    assert_eq!(suggestions[0].status, StatusSatisfiesLatest);
    assert_eq!(suggestions[1].status, StatusCurrent);
}

#[test]
fn latest_satisfying_range_is_current() {
    let suggestions = resolve_with_latest(
        vec![crate::support::tests::test_dependency("serde", "^1.0.0")],
        "1.5.0",
    );

    assert_eq!(suggestions[0].status, StatusSatisfiesLatest);
}
}
