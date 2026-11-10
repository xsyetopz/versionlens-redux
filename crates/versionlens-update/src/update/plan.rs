use std::cmp::Ordering;

use versionlens_model::{
    VersionDialect, compare_versions_for_dialect, latest_version_for_dialect,
    normalized_version_for_dialect, requirement_satisfies_latest_for_dialect,
};

use crate::support::minimum_version_token;

/// The update plan for one requirement. `update::plan` builds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub status: PlanStatus,
    pub bumps: Vec<Bump>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanStatus {
    /// The requirement already accepts the newest release.
    AllowsLatest,
    /// The requirement excludes the newest release.
    ExcludesLatest,
}

/// One candidate version for a requirement. Bumps are deduplicated by rendered text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bump {
    pub kind: BumpKind,
    pub version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpKind {
    Satisfying,
    Patch,
    Minor,
    Major,
    Latest,
}

/// Builds the status and the bounded bump set for `requirement`.
///
/// Candidates come from the registry's `versions`. Pre-releases are never
/// candidates. An exact pin (`=x`, `==x`, `[x]`, or a bare version in the
/// Semver dialect) targets its version. Cargo does not pin on a bare version.
pub fn plan(requirement: &str, dialect: VersionDialect, versions: &[String]) -> Plan {
    let requirement = requirement.trim();
    let Some(latest) =
        latest_version_for_dialect(versions.iter().map(String::as_str), false, &[], dialect)
    else {
        return Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![],
        };
    };

    let pin = exact_pin(requirement, dialect);
    let satisfies = |version: &str| match pin {
        Some(pinned) => {
            compare_versions_for_dialect(pinned, version, dialect) == Some(Ordering::Equal)
        }
        None => requirement_satisfies_latest_for_dialect(requirement, version, dialect),
    };
    let status = if satisfies(&latest) {
        PlanStatus::AllowsLatest
    } else {
        PlanStatus::ExcludesLatest
    };

    let releases: Vec<&str> = versions
        .iter()
        .map(String::as_str)
        .filter(|version| is_release(version, dialect) && !release_numbers(version).is_empty())
        .collect();

    let mut candidates: Vec<(BumpKind, &str)> = Vec::new();
    let satisfying = newest(
        releases
            .iter()
            .copied()
            .filter(|version| satisfies(version)),
        dialect,
    );
    if let Some(base) = satisfying {
        let base_numbers = release_numbers(base);
        let base_major = base_numbers.first().copied().unwrap_or(0);
        candidates.push((BumpKind::Satisfying, base));
        if let Some(patch) = newest_in_line(&releases, &base_numbers, 2, dialect) {
            candidates.push((BumpKind::Patch, patch));
        }
        if let Some(minor) = newest_in_line(&releases, &base_numbers, 1, dialect) {
            candidates.push((BumpKind::Minor, minor));
        }
        let next_major = releases
            .iter()
            .filter_map(|version| release_numbers(version).first().copied())
            .filter(|major| *major > base_major)
            .min();
        if let Some(major) = next_major
            && let Some(version) = newest_in_line(&releases, &[major], 1, dialect)
        {
            candidates.push((BumpKind::Major, version));
        }
    }
    candidates.push((BumpKind::Latest, latest.as_str()));

    let latest_text = rendered_text(&latest, dialect);
    let mut bumps: Vec<Bump> = Vec::new();
    for (kind, version) in candidates {
        let text = rendered_text(version, dialect);
        // Only the Latest bump names the latest release; the other tiers skip it.
        if text == latest_text && kind != BumpKind::Latest {
            continue;
        }
        if bumps
            .iter()
            .any(|bump| rendered_text(&bump.version, dialect) == text)
        {
            continue;
        }
        // A candidate whose text equals the requirement's own version changes nothing.
        if satisfies(version) && floor_reaches(requirement, version, dialect) {
            continue;
        }
        bumps.push(Bump {
            kind,
            version: version.to_owned(),
        });
    }

    Plan { status, bumps }
}

/// Whether `requirement` has a floor that an update raises while it allows latest.
///
/// Only a single `^`, `~` or Cargo bare comparator has one. Ranges such as `>=`
/// have no floor to raise, and a floor bump on a compound range (`^1 || ^2`,
/// `^1.2, <1.5`) would drop its other comparators. A registry alias
/// (`npm:chalk@^1.0.0`) is read by its range.
pub fn has_a_floor(requirement: &str, dialect: VersionDialect) -> bool {
    let requirement = requirement.trim();
    let requirement = versionlens_model::registry_alias_requirement(requirement)
        .unwrap_or(requirement)
        .trim();
    let single_comparator = !requirement
        .trim_start_matches(['^', '~', '>', '='])
        .trim_start()
        .contains(|character: char| character.is_whitespace() || matches!(character, ',' | '|'));
    single_comparator
        && (requirement.starts_with(['^', '~'])
            || (dialect == VersionDialect::Cargo
                && requirement.starts_with(|character: char| character.is_ascii_digit())))
}

/// Whether the floor of `requirement` already spells `version`, so raising the
/// floor to `version` changes nothing.
pub fn floor_reaches(requirement: &str, version: &str, dialect: VersionDialect) -> bool {
    minimum_version_token(requirement.trim()).is_some_and(|token| {
        rendered_text(token.trim_matches(['[', ']']), dialect) == rendered_text(version, dialect)
    })
}

/// Whether raising the floor of `requirement` to `version` changes its major
/// or minor number. A patch-only floor bump is too small to report as an
/// update, so it stays a lens and a code action.
pub fn floor_crosses_minor(requirement: &str, version: &str, dialect: VersionDialect) -> bool {
    has_a_floor(requirement, dialect)
        && minimum_version_token(requirement.trim())
            .and_then(major_minor)
            .zip(major_minor(version))
            .is_some_and(|(floor, version)| floor != version)
}

/// The major and minor numbers of a version, with a missing minor read as 0.
fn major_minor(version: &str) -> Option<(u64, u64)> {
    let mut numbers = version
        .trim_matches(['[', ']'])
        .split(|character: char| !character.is_ascii_digit());
    let major = numbers.next()?.parse().ok()?;
    let minor = numbers
        .next()
        .and_then(|minor| minor.parse().ok())
        .unwrap_or(0);
    Some((major, minor))
}

/// The version an exact pin names: `=x`, `==x`, `[x]`, or a bare version in the
/// Semver dialect, which reads a bare version as exact. The Cargo dialect reads a
/// bare version as a caret requirement, so it is not a pin there. Requirements
/// with other operators, ranges or wildcards are not pins.
fn exact_pin(requirement: &str, dialect: VersionDialect) -> Option<&str> {
    let pinned = if let Some(rest) = requirement
        .strip_prefix("==")
        .or_else(|| requirement.strip_prefix('='))
    {
        rest
    } else if let Some(rest) = requirement
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        rest
    } else if dialect == VersionDialect::Semver {
        requirement
    } else {
        return None;
    };
    let pinned = pinned.trim();
    let is_plain = !pinned.is_empty()
        && !pinned.contains(|character: char| {
            character.is_whitespace()
                || matches!(
                    character,
                    '<' | '>' | '^' | '~' | '*' | ',' | '|' | '=' | '[' | ']'
                )
        });
    (is_plain && normalized_version_for_dialect(pinned, dialect).is_some()).then_some(pinned)
}

/// The version as the dialect renders it, so `v1.5.0` and `1.5.0` are one text.
fn rendered_text(version: &str, dialect: VersionDialect) -> String {
    normalized_version_for_dialect(version, dialect).unwrap_or_else(|| version.trim().to_owned())
}

fn is_release(version: &str, dialect: VersionDialect) -> bool {
    latest_version_for_dialect([version], false, &[], dialect).is_some()
}

/// The numeric release segments before any pre-release or build suffix.
fn release_numbers(version: &str) -> Vec<u64> {
    let version = version.trim_start_matches(['v', 'V']);
    let core = version
        .split(|character: char| !(character.is_ascii_digit() || character == '.'))
        .next()
        .unwrap_or_default();
    core.split('.')
        .filter(|part| !part.is_empty())
        .map_while(|part| part.parse::<u64>().ok())
        .collect()
}

/// Whether the first `depth` segments of `numbers` equal those of `line`.
fn same_line(numbers: &[u64], line: &[u64], depth: usize) -> bool {
    (0..depth).all(|index| numbers.get(index).unwrap_or(&0) == line.get(index).unwrap_or(&0))
}

fn newest_in_line<'a>(
    releases: &[&'a str],
    line: &[u64],
    depth: usize,
    dialect: VersionDialect,
) -> Option<&'a str> {
    newest(
        releases
            .iter()
            .copied()
            .filter(|version| same_line(&release_numbers(version), line, depth)),
        dialect,
    )
}

fn newest<'a>(versions: impl Iterator<Item = &'a str>, dialect: VersionDialect) -> Option<&'a str> {
    versions.max_by(|left, right| {
        compare_versions_for_dialect(left, right, dialect).unwrap_or(Ordering::Equal)
    })
}

#[cfg(test)]
mod tests {
use super::{Bump, BumpKind, Plan, PlanStatus, plan};
use versionlens_model::VersionDialect::{Cargo, Pep440, Semver};

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
fn cargo_bare_version_is_a_caret_requirement() {
    let result = plan("1.0.200", Cargo, &versions(&["1.0.200", "1.0.210"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::AllowsLatest,
            bumps: vec![bump(BumpKind::Latest, "1.0.210")],
        }
    );
}

#[test]
fn cargo_exact_pin_names_only_its_version() {
    let result = plan("=1.0.200", Cargo, &versions(&["1.0.200", "1.0.210"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![bump(BumpKind::Latest, "1.0.210")],
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
fn bump_spelling_the_requirement_version_differently_is_dropped() {
    let result = plan("~1.5.0", Semver, &versions(&["1.5.0", "v1.5.0", "1.6.0"]));

    assert_eq!(
        result,
        Plan {
            status: PlanStatus::ExcludesLatest,
            bumps: vec![bump(BumpKind::Latest, "1.6.0")],
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
}
