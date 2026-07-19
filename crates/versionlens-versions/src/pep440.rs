use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

mod requirement;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Pep440Version {
    epoch: u64,
    release: Vec<u64>,
    pre: Option<Prerelease>,
    post: Option<u64>,
    dev: Option<u64>,
    local: Option<Vec<LocalSegment>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum PrereleaseKind {
    Alpha,
    Beta,
    ReleaseCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Prerelease {
    kind: PrereleaseKind,
    number: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalSegment {
    Text(String),
    Numeric(u64),
}

impl Ord for LocalSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Text(left), Self::Text(right)) => left.cmp(right),
            (Self::Numeric(left), Self::Numeric(right)) => left.cmp(right),
            (Self::Text(_), Self::Numeric(_)) => Ordering::Less,
            (Self::Numeric(_), Self::Text(_)) => Ordering::Greater,
        }
    }
}

impl PartialOrd for LocalSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pep440Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.epoch
            .cmp(&other.epoch)
            .then_with(|| compare_release(&self.release, &other.release))
            .then_with(|| compare_prerelease(self, other))
            .then_with(|| self.post.cmp(&other.post))
            .then_with(|| compare_dev(self.dev, other.dev))
            .then_with(|| self.local.cmp(&other.local))
    }
}

impl PartialOrd for Pep440Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for Pep440Version {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        if self.epoch != 0 {
            write!(formatter, "{}!", self.epoch)?;
        }
        for index in 0..self.release.len().max(3) {
            if index != 0 {
                formatter.write_str(".")?;
            }
            let part = self.release.get(index).copied().unwrap_or(0);
            write!(formatter, "{part}")?;
        }
        if let Some(pre) = self.pre {
            let marker = match pre.kind {
                PrereleaseKind::Alpha => "alpha",
                PrereleaseKind::Beta => "beta",
                PrereleaseKind::ReleaseCandidate => "rc",
            };
            write!(formatter, "-{marker}.{}", pre.number)?;
        }
        if let Some(post) = self.post {
            write!(formatter, ".post{post}")?;
        }
        if let Some(dev) = self.dev {
            write!(formatter, ".dev{dev}")?;
        }
        if let Some(local) = &self.local {
            formatter.write_str("+")?;
            for (index, part) in local.iter().enumerate() {
                if index != 0 {
                    formatter.write_str(".")?;
                }
                match part {
                    LocalSegment::Text(value) => formatter.write_str(value)?,
                    LocalSegment::Numeric(value) => write!(formatter, "{value}")?,
                }
            }
        }
        Ok(())
    }
}

pub(super) fn parse_version(raw: &str) -> Option<Pep440Version> {
    let raw = raw.trim();
    let raw = raw
        .strip_prefix('v')
        .or_else(|| raw.strip_prefix('V'))
        .unwrap_or(raw);
    let normalized = raw.to_ascii_lowercase();
    let (public, local) = normalized
        .split_once('+')
        .map_or((normalized.as_str(), None), |(public, local)| {
            (public, Some(local))
        });
    if local.is_some_and(|value| value.contains('+')) {
        return None;
    }

    let (epoch, public) = match public.split_once('!') {
        Some((epoch, public)) => (parse_digits(epoch)?, public),
        None => (0, public),
    };
    if public.contains('!') {
        return None;
    }

    let release_end = release_end(public)?;
    let release_raw = &public[..release_end];
    if release_raw.is_empty() || release_raw.starts_with('.') || release_raw.ends_with('.') {
        return None;
    }
    let release = release_raw
        .split('.')
        .map(parse_digits)
        .collect::<Option<Vec<_>>>()?;
    let mut suffix = &public[release_end..];

    let pre = parse_prerelease(&mut suffix).ok()?;
    let post = parse_postrelease(&mut suffix).ok()?;
    let dev = parse_devrelease(&mut suffix).ok()?;
    if !suffix.is_empty() {
        return None;
    }

    Some(Pep440Version {
        epoch,
        release,
        pre,
        post,
        dev,
        local: match local {
            Some(value) => Some(parse_local(value)?),
            None => None,
        },
    })
}

fn release_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    if index == 0 {
        return None;
    }
    while index < bytes.len()
        && bytes[index] == b'.'
        && bytes
            .get(index + 1)
            .is_some_and(|byte| byte.is_ascii_digit())
    {
        index += 1;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
    }
    Some(index)
}

fn parse_prerelease(suffix: &mut &str) -> Result<Option<Prerelease>, ()> {
    let candidate = trim_separator(suffix);
    let markers = [
        ("preview", PrereleaseKind::ReleaseCandidate),
        ("alpha", PrereleaseKind::Alpha),
        ("beta", PrereleaseKind::Beta),
        ("pre", PrereleaseKind::ReleaseCandidate),
        ("rc", PrereleaseKind::ReleaseCandidate),
        ("c", PrereleaseKind::ReleaseCandidate),
        ("a", PrereleaseKind::Alpha),
        ("b", PrereleaseKind::Beta),
    ];
    let Some((marker, kind)) = markers
        .into_iter()
        .find(|(marker, _)| candidate.starts_with(marker))
    else {
        return Ok(None);
    };
    let rest = trim_separator(&candidate[marker.len()..]);
    let (number, rest) = take_optional_number(rest).ok_or(())?;
    *suffix = rest;
    Ok(Some(Prerelease { kind, number }))
}

type OptionalReleaseNumber = Result<Option<u64>, ()>;

fn parse_postrelease(suffix: &mut &str) -> OptionalReleaseNumber {
    if let Some(rest) = suffix.strip_prefix('-')
        && rest.starts_with(|char: char| char.is_ascii_digit())
    {
        let (number, rest) = take_number(rest).ok_or(())?;
        *suffix = rest;
        return Ok(Some(number));
    }

    let candidate = trim_separator(suffix);
    let Some(marker) = ["post", "rev", "r"]
        .into_iter()
        .find(|marker| candidate.starts_with(marker))
    else {
        return Ok(None);
    };
    let rest = trim_separator(&candidate[marker.len()..]);
    let (number, rest) = take_optional_number(rest).ok_or(())?;
    *suffix = rest;
    Ok(Some(number))
}

fn parse_devrelease(suffix: &mut &str) -> OptionalReleaseNumber {
    let candidate = trim_separator(suffix);
    let Some(rest) = candidate.strip_prefix("dev") else {
        return Ok(None);
    };
    let (number, rest) = take_optional_number(trim_separator(rest)).ok_or(())?;
    *suffix = rest;
    Ok(Some(number))
}

fn take_optional_number(value: &str) -> Option<(u64, &str)> {
    if value.starts_with(|char: char| char.is_ascii_digit()) {
        take_number(value)
    } else {
        Some((0, value))
    }
}

fn take_number(value: &str) -> Option<(u64, &str)> {
    let end = value
        .find(|char: char| !char.is_ascii_digit())
        .unwrap_or(value.len());
    Some((parse_digits(&value[..end])?, &value[end..]))
}

fn parse_digits(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn trim_separator(value: &str) -> &str {
    value.trim_start_matches(['.', '-', '_'])
}

fn parse_local(value: &str) -> Option<Vec<LocalSegment>> {
    let parts = value
        .split(['.', '-', '_'])
        .map(|part| {
            if part.is_empty() || !part.chars().all(|char| char.is_ascii_alphanumeric()) {
                return None;
            }
            Some(match part.parse::<u64>() {
                Ok(number) => LocalSegment::Numeric(number),
                Err(_) => LocalSegment::Text(part.to_owned()),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    (!parts.is_empty()).then_some(parts)
}

fn compare_release(left: &[u64], right: &[u64]) -> Ordering {
    let count = left.len().max(right.len());
    (0..count)
        .map(|index| {
            left.get(index)
                .copied()
                .unwrap_or(0)
                .cmp(&right.get(index).copied().unwrap_or(0))
        })
        .find(|ordering| !ordering.is_eq())
        .unwrap_or(Ordering::Equal)
}

fn compare_prerelease(left: &Pep440Version, right: &Pep440Version) -> Ordering {
    match (left.pre, right.pre) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => {
            if right.dev.is_some() && right.post.is_none() {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (None, Some(_)) => {
            if left.dev.is_some() && left.post.is_none() {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
        (None, None) => Ordering::Equal,
    }
}

fn compare_dev(left: Option<u64>, right: Option<u64>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

pub(super) fn prerelease_allowed(
    version: &Pep440Version,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> bool {
    if version.pre.is_none() && version.dev.is_none() {
        return true;
    }
    if !include_prereleases {
        return false;
    }
    if prerelease_tags.is_empty() {
        return true;
    }

    let tag = if let Some(pre) = version.pre {
        match pre.kind {
            PrereleaseKind::Alpha => "a",
            PrereleaseKind::Beta => "b",
            PrereleaseKind::ReleaseCandidate => "rc",
        }
    } else {
        "dev"
    };
    prerelease_tags.iter().any(|candidate| {
        candidate.eq_ignore_ascii_case(tag)
            || (tag == "a" && candidate.eq_ignore_ascii_case("alpha"))
            || (tag == "b" && candidate.eq_ignore_ascii_case("beta"))
    })
}

pub(super) fn requirement_satisfies(requirement: &str, latest: &str) -> bool {
    requirement::requirement_satisfies(requirement, latest).unwrap_or(false)
}

pub(super) fn requirement_is_parseable(requirement: &str) -> bool {
    requirement.trim().is_empty() || requirement::parse_requirement(requirement).is_some()
}

pub(super) fn is_update_available(latest: &str, requirement: &str) -> bool {
    if requirement.trim().is_empty() {
        return parse_version(latest).is_some();
    }
    requirement::is_update_available(latest, requirement).unwrap_or(false)
}
