use std::cmp::Ordering;

use super::{Pep440Version, compare_release, parse_version};

#[derive(Debug, Clone, Copy)]
enum Operator {
    ArbitraryEqual,
    Compatible,
    Equal,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
}

#[derive(Debug, Clone)]
pub(super) struct Specifier {
    operator: Operator,
    expected: ExpectedVersion,
}

#[derive(Debug, Clone)]
enum ExpectedVersion {
    Version(Pep440Version),
    Prefix { epoch: u64, release: Vec<u64> },
    Arbitrary(String),
}

pub(super) fn parse_requirement(requirement: &str) -> Option<Vec<Specifier>> {
    let parts = requirement_parts(requirement)?;
    let specifiers = parts
        .iter()
        .map(|part| parse_specifier(part))
        .collect::<Option<Vec<_>>>()?;
    (!specifiers.is_empty()).then_some(specifiers)
}

pub(super) fn requirement_satisfies(requirement: &str, candidate: &str) -> Option<bool> {
    if requirement.trim().is_empty() {
        return Some(true);
    }
    let candidate_raw = candidate.trim();
    let candidate = parse_version(candidate_raw)?;
    let specifiers = parse_requirement(requirement)?;
    Some(
        specifiers
            .iter()
            .all(|specifier| specifier_matches(specifier, &candidate, candidate_raw)),
    )
}

pub(super) fn is_update_available(latest: &str, requirement: &str) -> Option<bool> {
    let latest = parse_version(latest)?;
    let specifiers = parse_requirement(requirement)?;
    if lower_bound(&specifiers).is_some_and(|lower| latest < lower) {
        return Some(false);
    }
    let satisfies = specifiers
        .iter()
        .all(|specifier| specifier_matches(specifier, &latest, &latest.to_string()));
    if satisfies {
        return Some(false);
    }

    let exact = specifiers.iter().find_map(exact_version);
    Some(exact.is_none_or(|current| latest > *current))
}

fn requirement_parts(requirement: &str) -> Option<Vec<String>> {
    let mut output = Vec::new();
    for comma_part in requirement.split(',') {
        let tokens = comma_part
            .split_whitespace()
            .filter(|token| !token.eq_ignore_ascii_case("and"))
            .collect::<Vec<_>>();
        let mut index = 0;
        while index < tokens.len() {
            let token = tokens[index];
            if is_operator(token) {
                let version = tokens.get(index + 1)?;
                output.push(format!("{token}{version}"));
                index += 2;
            } else {
                output.push(token.to_owned());
                index += 1;
            }
        }
    }
    Some(output)
}

fn is_operator(value: &str) -> bool {
    matches!(value, "===" | "~=" | "==" | "!=" | "<=" | ">=" | "<" | ">")
}

fn parse_specifier(raw: &str) -> Option<Specifier> {
    let raw = raw.trim();
    let (operator, version) = [
        ("===", Operator::ArbitraryEqual),
        ("~=", Operator::Compatible),
        ("==", Operator::Equal),
        ("!=", Operator::NotEqual),
        ("<=", Operator::LessEqual),
        (">=", Operator::GreaterEqual),
        ("<", Operator::Less),
        (">", Operator::Greater),
    ]
    .into_iter()
    .find_map(|(prefix, operator)| raw.strip_prefix(prefix).map(|rest| (operator, rest)))
    .unwrap_or((Operator::Equal, raw));
    let version = version.trim();
    if version.is_empty() {
        return None;
    }

    let expected = if matches!(operator, Operator::ArbitraryEqual) {
        ExpectedVersion::Arbitrary(version.to_ascii_lowercase())
    } else if matches!(operator, Operator::Equal | Operator::NotEqual) && version.ends_with(".*") {
        let (epoch, release) = parse_prefix(&version[..version.len() - 2])?;
        ExpectedVersion::Prefix { epoch, release }
    } else {
        ExpectedVersion::Version(parse_version(version)?)
    };

    if matches!(operator, Operator::Compatible)
        && matches!(&expected, ExpectedVersion::Version(version) if version.release.len() < 2)
    {
        return None;
    }
    if matches!(
        operator,
        Operator::Compatible
            | Operator::LessEqual
            | Operator::GreaterEqual
            | Operator::Less
            | Operator::Greater
    ) && matches!(&expected, ExpectedVersion::Version(version) if version.local.is_some())
    {
        return None;
    }

    Some(Specifier { operator, expected })
}

fn parse_prefix(raw: &str) -> Option<(u64, Vec<u64>)> {
    let (epoch, release) = match raw.split_once('!') {
        Some((epoch, release)) => (epoch.parse().ok()?, release),
        None => (0, raw),
    };
    let release = release
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect::<Option<Vec<_>>>()?;
    (!release.is_empty()).then_some((epoch, release))
}

fn specifier_matches(
    specifier: &Specifier,
    candidate: &Pep440Version,
    candidate_raw: &str,
) -> bool {
    match (&specifier.operator, &specifier.expected) {
        (Operator::ArbitraryEqual, ExpectedVersion::Arbitrary(expected)) => {
            candidate_raw.eq_ignore_ascii_case(expected)
        }
        (Operator::Equal, ExpectedVersion::Prefix { epoch, release }) => {
            prefix_matches(candidate, *epoch, release)
        }
        (Operator::NotEqual, ExpectedVersion::Prefix { epoch, release }) => {
            !prefix_matches(candidate, *epoch, release)
        }
        (Operator::Equal, ExpectedVersion::Version(expected)) => public_equal(candidate, expected),
        (Operator::NotEqual, ExpectedVersion::Version(expected)) => {
            !public_equal(candidate, expected)
        }
        (Operator::LessEqual, ExpectedVersion::Version(expected)) => {
            public_cmp(candidate, expected).is_le()
        }
        (Operator::GreaterEqual, ExpectedVersion::Version(expected)) => {
            public_cmp(candidate, expected).is_ge()
        }
        (Operator::Less, ExpectedVersion::Version(expected)) => {
            public_cmp(candidate, expected).is_lt()
                && !((candidate.pre.is_some() || candidate.dev.is_some())
                    && expected.pre.is_none()
                    && expected.dev.is_none()
                    && same_base_version(candidate, expected))
        }
        (Operator::Greater, ExpectedVersion::Version(expected)) => {
            public_cmp(candidate, expected).is_gt()
                && !(candidate.post.is_some()
                    && expected.post.is_none()
                    && same_base_version(candidate, expected))
        }
        (Operator::Compatible, ExpectedVersion::Version(expected)) => {
            public_cmp(candidate, expected).is_ge()
                && public_cmp(candidate, &compatible_upper_bound(expected)).is_lt()
        }
        _ => false,
    }
}

fn public_cmp(candidate: &Pep440Version, expected: &Pep440Version) -> Ordering {
    let mut candidate = candidate.clone();
    candidate.local = None;
    let mut expected = expected.clone();
    expected.local = None;
    candidate.cmp(&expected)
}

fn public_equal(candidate: &Pep440Version, expected: &Pep440Version) -> bool {
    if expected.local.is_some() {
        return candidate.cmp(expected).is_eq();
    }
    let mut candidate = candidate.clone();
    candidate.local = None;
    candidate.cmp(expected).is_eq()
}

fn same_base_version(candidate: &Pep440Version, expected: &Pep440Version) -> bool {
    candidate.epoch == expected.epoch
        && compare_release(&candidate.release, &expected.release).is_eq()
}

fn prefix_matches(candidate: &Pep440Version, epoch: u64, release: &[u64]) -> bool {
    candidate.epoch == epoch
        && candidate.release.len() >= release.len()
        && candidate.release[..release.len()] == *release
}

fn compatible_upper_bound(version: &Pep440Version) -> Pep440Version {
    let mut upper = version.clone();
    let index = upper.release.len() - 2;
    upper.release[index] = upper.release[index].saturating_add(1);
    upper.release.truncate(index + 1);
    upper.pre = None;
    upper.post = None;
    upper.dev = None;
    upper.local = None;
    upper
}

fn lower_bound(specifiers: &[Specifier]) -> Option<Pep440Version> {
    specifiers
        .iter()
        .filter_map(
            |specifier| match (&specifier.operator, &specifier.expected) {
                (
                    Operator::Compatible
                    | Operator::Equal
                    | Operator::GreaterEqual
                    | Operator::Greater,
                    ExpectedVersion::Version(version),
                ) => Some(version.clone()),
                _ => None,
            },
        )
        .max()
}

fn exact_version(specifier: &Specifier) -> Option<&Pep440Version> {
    match (&specifier.operator, &specifier.expected) {
        (Operator::Equal, ExpectedVersion::Version(version)) => Some(version),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::requirement_satisfies;

    #[test]
    fn exclusive_ordered_comparisons_exclude_same_base_derived_releases() {
        for (requirement, candidate, expected) in [
            (">1.0", "1.0.post1", false),
            (">1.0.post1", "1.0.post2", true),
            ("<1.0", "1.0rc1", false),
            ("<1.0rc1", "1.0b1", true),
        ] {
            assert_eq!(
                requirement_satisfies(requirement, candidate),
                Some(expected),
                "{requirement} against {candidate}",
            );
        }
    }
}
