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

/// Whether `requirement` has a floor that an update raises while it still allows
/// latest: `^`, `~`, or a Cargo bare version. Other ranges, such as `>=`, have
/// no floor to raise.
pub fn has_a_floor(requirement: &str, dialect: VersionDialect) -> bool {
    let requirement = requirement.trim();
    requirement.starts_with(['^', '~'])
        || (dialect == VersionDialect::Cargo
            && requirement.starts_with(|character: char| character.is_ascii_digit()))
}

/// Whether the floor of `requirement` already spells `version`, so raising the
/// floor to `version` changes nothing.
pub fn floor_reaches(requirement: &str, version: &str, dialect: VersionDialect) -> bool {
    minimum_version_token(requirement.trim()).is_some_and(|token| {
        rendered_text(token.trim_matches(['[', ']']), dialect) == rendered_text(version, dialect)
    })
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
mod tests;
