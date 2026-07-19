use crate::parse::{normalize_requirement, parse_coerced_version, parse_version};
use crate::pep440;
use crate::policy::UpdateLevel;
use crate::policy::VersionDialect;

use super::compare::is_newer;
use super::requirement_has_empty_comparator_intersection;
use super::requirements::{
    disjunctive_requirement_satisfies, nuget_requirement_satisfies, pessimistic_update_available,
    range_requirement_satisfies, semver_range_requirement_satisfies,
};
use crate::policy::UpdateLevel::{Major, Minor, Patch};

pub fn is_update_available(latest: &str, requirement: &str) -> bool {
    if requirement.trim().is_empty() {
        return true;
    }
    if latest_is_older_than_semver_lower_bound(latest, requirement) {
        return false;
    }
    if let Some(is_update) = pessimistic_update_available(requirement, latest) {
        return is_update;
    }
    if let Some(satisfies) = nuget_requirement_satisfies(requirement, latest) {
        return !satisfies;
    }
    if let Some(satisfies) = disjunctive_requirement_satisfies(requirement, latest) {
        return parse_version(latest).is_some() && !satisfies;
    }
    if requirement_has_empty_comparator_intersection(requirement) {
        return parse_version(latest).is_some();
    }

    if let Some(satisfies) = semver_range_requirement_satisfies(requirement, latest) {
        return !satisfies;
    }

    if range_requirement_satisfies(requirement, latest) {
        return false;
    }
    is_newer(latest, requirement)
}

pub fn is_update_available_for_dialect(
    latest: &str,
    requirement: &str,
    dialect: VersionDialect,
) -> bool {
    match dialect {
        VersionDialect::Semver => is_update_available(latest, requirement),
        VersionDialect::Pep440 => pep440::is_update_available(latest, requirement),
    }
}

pub fn requirement_satisfies_latest(requirement: &str, latest: &str) -> bool {
    range_requirement_satisfies(requirement, latest)
}

pub fn requirement_satisfies_latest_for_dialect(
    requirement: &str,
    latest: &str,
    dialect: VersionDialect,
) -> bool {
    match dialect {
        VersionDialect::Semver => requirement_satisfies_latest(requirement, latest),
        VersionDialect::Pep440 => pep440::requirement_satisfies(requirement, latest),
    }
}

pub fn requirement_is_parseable(requirement: &str, latest: &str) -> bool {
    let requirement = requirement.trim();
    requirement.is_empty()
        || parse_version(requirement).is_some()
        || pessimistic_update_available(requirement, latest).is_some()
        || nuget_requirement_satisfies(requirement, latest).is_some()
        || disjunctive_requirement_satisfies(requirement, latest).is_some()
        || semver_range_requirement_satisfies(requirement, latest).is_some()
}

pub fn requirement_is_parseable_for_dialect(
    requirement: &str,
    latest: &str,
    dialect: VersionDialect,
) -> bool {
    match dialect {
        VersionDialect::Semver => requirement_is_parseable(requirement, latest),
        VersionDialect::Pep440 => pep440::requirement_is_parseable(requirement),
    }
}

fn latest_is_older_than_semver_lower_bound(latest: &str, requirement: &str) -> bool {
    if requirement.contains("||") {
        return false;
    }
    let Some(latest) = parse_version(latest) else {
        return false;
    };
    semver_lower_bounds(requirement)
        .into_iter()
        .max()
        .is_some_and(|lower| latest < lower)
}

fn semver_lower_bounds(requirement: &str) -> Vec<semver::Version> {
    normalize_requirement(requirement)
        .split(|char: char| char.is_whitespace() || char == ',')
        .filter_map(|part| {
            let part = part.trim();
            if part.is_empty() || part.starts_with('<') {
                return None;
            }
            let version = part.trim_start_matches(['^', '~', '>', '=']);
            (!version.contains(['*', 'x', 'X']))
                .then(|| parse_version(version))
                .flatten()
        })
        .collect()
}

pub fn is_build_update(latest: &str, requirement: &str) -> bool {
    let Some(latest) = parse_version(latest) else {
        return false;
    };
    let Some(current) = parse_version(requirement) else {
        return false;
    };

    latest.major == current.major
        && latest.minor == current.minor
        && latest.patch == current.patch
        && latest.pre == current.pre
        && latest.build != current.build
}

pub fn build_variants<'a>(
    requirement: &str,
    versions: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let Some(current) = parse_version(requirement) else {
        return vec![];
    };

    let mut variants = vec![];
    for version in versions {
        if parse_version(version)
            .or_else(|| parse_coerced_version(version))
            .is_some_and(|candidate| {
                candidate.major == current.major
                    && candidate.minor == current.minor
                    && candidate.patch == current.patch
                    && candidate.pre == current.pre
            })
            && !variants.iter().any(|variant| variant == version)
        {
            variants.push(version.to_owned());
        }
    }

    if variants.len() > 1 { variants } else { vec![] }
}

pub fn update_level(latest: &str, requirement: &str) -> Option<UpdateLevel> {
    let latest = parse_version(latest)?;
    let current = parse_version(requirement).or_else(|| parse_coerced_version(requirement))?;

    if latest.major != current.major {
        Some(Major)
    } else if latest.minor != current.minor {
        Some(Minor)
    } else if latest.patch != current.patch {
        Some(Patch)
    } else {
        None
    }
}
