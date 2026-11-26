//! Lenient version numbers, requirement ranges, and update plans shared by every registry.

use std::cmp::Ordering;

/// What a bare version, written without an operator, means in a registry's requirement syntax.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Bare {
    /// `1.2.3` pins exactly (npm, Composer, Pub, Maven).
    Exact,
    /// `1.2.3` means `^1.2.3` (Cargo, Hex, Julia).
    Caret,
    /// `1.2.3` means `>=1.2.3` (`NuGet`).
    Min,
    /// `1.2` means any `1.2.*` (Docker tags, GitHub refs).
    Prefix,
}

/// A parsed version: numeric segments, a lowercase tag, and the text it came from.
#[derive(Clone, Debug)]
pub(crate) struct Version {
    pub(crate) nums: Vec<u64>,
    pub(crate) tag: String,
    pub(crate) raw: String,
}

const PRERELEASE_WORDS: &[&str] = &[
    "a",
    "b",
    "c",
    "m",
    "rc",
    "cr",
    "alpha",
    "beta",
    "pre",
    "preview",
    "dev",
    "snapshot",
    "canary",
    "next",
    "nightly",
    "insiders",
    "experimental",
    "ea",
    "milestone",
    "unstable",
    "test",
    "edge",
];

/// Splits `v1.2.3-rc.1` into the `v` prefix, the numeric head `1.2.3`, and the suffix `-rc.1`.
pub(crate) fn split(text: &str) -> (&str, &str, &str) {
    let skip = usize::from(
        text.starts_with(['v', 'V']) && text[1..].starts_with(|c: char| c.is_ascii_digit()),
    );
    let rest = &text[skip..];
    let numeric = |from: usize| {
        rest[from..]
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .map_or(rest.len(), |end| from + end)
    };
    let mut end = numeric(0);
    // A date, `2024-01-31` or vcpkg's `2023-08-09.1`, is a release and not a `2024` prerelease:
    // learn.microsoft.com/en-us/vcpkg/reference/vcpkg-json#version-date, and Stackage nightlies.
    let date = rest.as_bytes().get(..10).is_some_and(|d| {
        d.iter().enumerate().all(|(i, b)| match i {
            4 | 7 => *b == b'-',
            _ => b.is_ascii_digit(),
        })
    });
    if end == 4 && date {
        end = numeric(10);
    }
    while rest[..end].ends_with('.') {
        end -= 1;
    }
    (&text[..skip], &rest[..end], &rest[end..])
}

impl Version {
    pub(crate) fn parse(raw: &str) -> Option<Self> {
        let raw = raw.trim();
        let (_, head, suffix) = split(raw);
        if head.is_empty() {
            return None;
        }
        let nums = head
            .split(['.', '-'])
            .map(|n| n.parse().ok())
            .collect::<Option<Vec<u64>>>()?;
        let tag = suffix
            .split('+')
            .next()
            .unwrap_or_default()
            .trim_start_matches(['-', '.', '_'])
            .to_ascii_lowercase();
        let tag = if matches!(tag.as_str(), "final" | "release" | "ga") {
            String::new()
        } else {
            tag
        };
        Some(Self {
            nums,
            tag,
            raw: raw.to_owned(),
        })
    }

    fn num(&self, index: usize) -> u64 {
        self.nums.get(index).copied().unwrap_or(0)
    }

    pub(crate) fn is_prerelease(&self) -> bool {
        let word: String = self
            .tag
            .chars()
            .take_while(char::is_ascii_alphabetic)
            .collect();
        !self.tag.is_empty() && (word.is_empty() || PRERELEASE_WORDS.contains(&word.as_str()))
    }

    /// The build flavour a tag names, such as `alpine` or `jre`; updates keep the current flavour.
    pub(crate) fn variant(&self) -> String {
        if self.is_prerelease() {
            String::new()
        } else {
            self.tag.chars().filter(char::is_ascii_alphabetic).collect()
        }
    }

    fn bump(&self, index: usize) -> Self {
        let mut nums: Vec<u64> = (0..index).map(|i| self.num(i)).collect();
        nums.push(self.num(index).saturating_add(1));
        Self {
            nums,
            tag: String::new(),
            raw: String::new(),
        }
    }

    /// Renders this version in the style of `like`: same `v` prefix and, unless `precise`, the same
    /// number of segments (`^1.2` stays two segments) when that still names a different version.
    pub(crate) fn render(&self, like: &str, precise: bool) -> String {
        let (prefix, head, like_suffix) = split(like);
        let (_, new_head, suffix) = split(&self.raw);
        let count = head.split('.').count();
        let wildcard = like_suffix.starts_with(".x")
            || like_suffix.starts_with(".*")
            || like_suffix.starts_with(".X");
        let short: Vec<&str> = new_head.split('.').take(count).collect();
        let short = short.join(".");
        if wildcard {
            return format!("{prefix}{short}{like_suffix}");
        }
        let body = if !precise && !head.is_empty() && short != head {
            short
        } else {
            new_head.to_owned()
        };
        format!("{prefix}{body}{suffix}")
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Version {}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let len = self.nums.len().max(other.nums.len());
        (0..len)
            .map(|i| self.num(i).cmp(&other.num(i)))
            .find(|o| o.is_ne())
            .unwrap_or_else(|| match (self.is_prerelease(), other.is_prerelease()) {
                (false, true) => Ordering::Greater,
                (true, false) => Ordering::Less,
                _ => natural(&self.tag, &other.tag),
            })
    }
}

fn chunks(text: &str) -> impl Iterator<Item = &str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        rest = rest.trim_start_matches(|c: char| !c.is_ascii_alphanumeric());
        let digit = rest.chars().next()?.is_ascii_digit();
        let end = rest
            .find(|c: char| !c.is_ascii_alphanumeric() || c.is_ascii_digit() != digit)
            .unwrap_or(rest.len());
        let (chunk, tail) = rest.split_at(end);
        rest = tail;
        Some(chunk)
    })
}

/// Compares tags the way people read them: `rc.10` after `rc.9`, numbers before words.
fn natural(a: &str, b: &str) -> Ordering {
    let (mut left, mut right) = (chunks(a), chunks(b));
    loop {
        let order = match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => match (x.parse::<u64>(), y.parse::<u64>()) {
                (Ok(lhs), Ok(rhs)) => lhs.cmp(&rhs),
                (Ok(_), Err(_)) => Ordering::Less,
                (Err(_), Ok(_)) => Ordering::Greater,
                _ => x.cmp(y),
            },
        };
        if order.is_ne() {
            return order;
        }
    }
}

mod plan;
mod req;

pub(crate) use plan::{Plan, increments, level, plan};
pub(crate) use req::Req;

#[cfg(test)]
mod tests;
