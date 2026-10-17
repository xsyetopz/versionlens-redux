use super::{Bump, BumpKind, Plan, PlanStatus, plan};
use versionlens_model::VersionDialect::{Pep440, Semver};

fn versions(list: &[&str]) -> Vec<String> {
    list.iter().map(|version| (*version).to_owned()).collect()
}

fn bump(kind: BumpKind, version: &str) -> Bump {
    Bump {
        kind,
        version: version.to_owned(),
    }
}

#[test]
fn caret_range_behind_latest_allows_latest_and_bumps_to_latest() {
    let result = plan(
        "^2.5.8",
        Semver,
        &versions(&["2.5.8", "2.5.9", "2.5.10", "2.5.14"]),
    );

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![bump(BumpKind::Latest, "2.5.14")],
        }
    );
}

#[test]
fn intermediate_releases_are_not_listed_individually() {
    let result = plan(
        "^0.41.7",
        Semver,
        &versions(&[
            "0.41.7", "0.41.11", "0.42.0", "0.42.1", "0.42.2", "0.42.3", "0.42.4", "0.42.5",
            "0.42.6",
        ]),
    );

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![
                bump(BumpKind::Satisfying, "0.41.11"),
                bump(BumpKind::Latest, "0.42.6"),
            ],
        }
    );
}

#[test]
fn prefix_similar_minimum_is_not_treated_as_latest() {
    let result = plan("^1.2.1", Semver, &versions(&["1.2.1", "1.2.10"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![bump(BumpKind::Latest, "1.2.10")],
        }
    );
}

#[test]
fn cargo_exact_pin_at_latest_offers_nothing() {
    let result = plan("=0.7.0", Semver, &versions(&["0.6.0", "0.7.0"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![],
        }
    );
}

#[test]
fn python_exact_pin_at_latest_offers_nothing() {
    let result = plan("==2.34.2", Pep440, &versions(&["2.34.1", "2.34.2"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![],
        }
    );
}

#[test]
fn bracketed_exact_pin_behind_latest_bumps_to_latest() {
    let result = plan("[1.0.0]", Semver, &versions(&["1.0.0", "1.0.1"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![bump(BumpKind::Latest, "1.0.1")],
        }
    );
}

#[test]
fn bare_npm_version_is_an_exact_pin() {
    let result = plan(
        "1.0.0",
        Semver,
        &versions(&["1.0.0", "1.0.1", "1.1.0", "2.0.0"]),
    );

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![
                bump(BumpKind::Patch, "1.0.1"),
                bump(BumpKind::Minor, "1.1.0"),
                bump(BumpKind::Latest, "2.0.0"),
            ],
        }
    );
}

#[test]
fn newest_major_line_is_offered_between_satisfying_and_latest() {
    let result = plan(
        "^1.0.0",
        Semver,
        &versions(&["1.0.0", "1.5.0", "2.0.0", "2.4.0", "3.0.0"]),
    );

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![
                bump(BumpKind::Satisfying, "1.5.0"),
                bump(BumpKind::Major, "2.4.0"),
                bump(BumpKind::Latest, "3.0.0"),
            ],
        }
    );
}

#[test]
fn requirement_already_at_latest_offers_no_bump() {
    let result = plan("^2.5.14", Semver, &versions(&["2.5.8", "2.5.14"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![],
        }
    );
}

#[test]
fn prerelease_versions_are_never_bumps() {
    let result = plan(
        "^1.0.0",
        Semver,
        &versions(&["1.0.0", "1.1.0", "2.0.0-beta.1"]),
    );

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![bump(BumpKind::Latest, "1.1.0")],
        }
    );
}

#[test]
fn no_releases_allows_latest_without_bumps() {
    let result = plan("^1.0.0", Semver, &[]);

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![],
        }
    );
}
