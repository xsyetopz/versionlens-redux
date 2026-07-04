use crate::model::UpdateLevel;
use crate::parse::{parse_coerced_version, parse_version};

use super::compare::is_newer;
use super::requirement_has_empty_comparator_intersection;
use super::requirements::{
    disjunctive_requirement_satisfies, nuget_requirement_satisfies, pessimistic_update_available,
    range_requirement_satisfies, semver_range_requirement_satisfies,
};

pub fn is_update_available(latest: &str, requirement: &str) -> bool {
    if requirement.trim().is_empty() {
        return true;
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

pub fn requirement_satisfies_latest(requirement: &str, latest: &str) -> bool {
    range_requirement_satisfies(requirement, latest)
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
        return Vec::new();
    };

    let mut variants = Vec::new();
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

    if variants.len() > 1 {
        variants
    } else {
        Vec::new()
    }
}

pub fn update_level(latest: &str, requirement: &str) -> Option<UpdateLevel> {
    let latest = parse_version(latest)?;
    let current = parse_version(requirement).or_else(|| parse_coerced_version(requirement))?;

    if latest.major != current.major {
        Some(UpdateLevel::Major)
    } else if latest.minor != current.minor {
        Some(UpdateLevel::Minor)
    } else if latest.patch != current.patch {
        Some(UpdateLevel::Patch)
    } else {
        None
    }
}
