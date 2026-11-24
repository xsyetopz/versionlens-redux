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
    let mut end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(rest.len());
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
            .split('.')
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Caret,
    Tilde,
    Pess,
    Prefix,
}

#[derive(Clone, Debug)]
struct Cmp {
    op: Op,
    v: Version,
}

impl Cmp {
    fn matches(&self, v: &Version) -> bool {
        let c = &self.v;
        let parts = c.nums.len();
        match self.op {
            Op::Eq => v == c,
            Op::Ne => v != c,
            Op::Lt => v < c,
            Op::Le => v <= c,
            Op::Gt => v > c,
            Op::Ge => v >= c,
            Op::Caret => {
                let index = c.nums.iter().position(|n| *n != 0).unwrap_or(parts - 1);
                v >= c && *v < c.bump(index)
            }
            Op::Tilde => v >= c && *v < c.bump(usize::from(parts >= 2)),
            Op::Pess => v >= c && *v < c.bump(parts.saturating_sub(2)),
            Op::Prefix => (0..parts).all(|i| v.num(i) == c.num(i)),
        }
    }
}

const OPERATORS: &[(&str, Op)] = &[
    ("===", Op::Eq),
    ("==", Op::Eq),
    ("!=", Op::Ne),
    ("~>", Op::Pess),
    ("~=", Op::Pess),
    (">=", Op::Ge),
    ("<=", Op::Le),
    ("=", Op::Eq),
    (">", Op::Gt),
    ("<", Op::Lt),
    ("^", Op::Caret),
    ("~", Op::Tilde),
];

/// A requirement: any of several alternatives, each a set of comparators that must all hold.
#[derive(Clone, Debug)]
pub(crate) struct Req {
    alts: Vec<Vec<Cmp>>,
}

impl Req {
    /// Parses npm, Cargo, PEP 440, `RubyGems`, Composer, and NuGet/Maven bracket syntax.
    pub(crate) fn parse(text: &str, bare: Bare) -> Option<Self> {
        let text = text.trim();
        if matches!(text, "" | "*" | "x" | "X" | "latest" | "any") {
            return Some(Self {
                alts: vec![Vec::new()],
            });
        }
        if text.starts_with(['[', '(']) {
            return bracket(text).map(|alt| Self { alts: vec![alt] });
        }
        let alts = text
            .split("||")
            .map(|alt| alternative(alt, bare))
            .collect::<Option<Vec<_>>>()?;
        Some(Self { alts })
    }

    pub(crate) fn matches(&self, v: &Version) -> bool {
        self.alts.iter().any(|alt| alt.iter().all(|c| c.matches(v)))
    }

    /// The version the requirement starts from: what is installed, or the oldest it allows.
    pub(crate) fn floor(&self) -> Option<&Version> {
        self.alts
            .first()?
            .iter()
            .find(|c| !matches!(c.op, Op::Ne | Op::Lt | Op::Le))
            .map(|c| &c.v)
    }
}

fn alternative(text: &str, bare: Bare) -> Option<Vec<Cmp>> {
    if let Some((low, high)) = text.split_once(" - ") {
        return Some(vec![
            Cmp {
                op: Op::Ge,
                v: Version::parse(low)?,
            },
            Cmp {
                op: Op::Le,
                v: Version::parse(high)?,
            },
        ]);
    }
    let mut cmps = Vec::new();
    let mut pending = "";
    for token in text
        .split([',', ' ', '\t'])
        .filter(|t| !t.is_empty() && *t != "and")
    {
        let (op, rest) = OPERATORS
            .iter()
            .find_map(|(symbol, op)| token.strip_prefix(symbol).map(|rest| (Some(*op), rest)))
            .unwrap_or((None, token));
        if rest.is_empty() {
            pending = token;
            continue;
        }
        let op = op.or_else(|| {
            OPERATORS
                .iter()
                .find(|(symbol, _)| *symbol == pending)
                .map(|(_, op)| *op)
        });
        pending = "";
        cmps.push(comparator(op, rest, bare)?);
    }
    Some(cmps)
}

fn comparator(op: Option<Op>, text: &str, bare: Bare) -> Option<Cmp> {
    let wild = text
        .find(['x', 'X', '*'])
        .filter(|i| text[..*i].is_empty() || text[..*i].ends_with('.'));
    if let Some(index) = wild {
        let head = text[..index].trim_end_matches('.');
        if head.is_empty() {
            return Some(Cmp {
                op: Op::Ge,
                v: Version::parse("0")?,
            });
        }
        let v = Version::parse(head)?;
        return Some(Cmp {
            op: if matches!(op, None | Some(Op::Eq)) {
                Op::Prefix
            } else {
                op?
            },
            v,
        });
    }
    let v = Version::parse(text)?;
    let op = op.unwrap_or(match bare {
        Bare::Exact => Op::Eq,
        Bare::Caret => Op::Caret,
        Bare::Min => Op::Ge,
        Bare::Prefix => Op::Prefix,
    });
    Some(Cmp { op, v })
}

/// `NuGet` and Maven ranges: `[1.0]`, `[1.0,2.0)`, `(,2.0]`.
fn bracket(text: &str) -> Option<Vec<Cmp>> {
    let text = text.split("],").next().unwrap_or(text);
    let inclusive_low = text.starts_with('[');
    let inclusive_high = !text.ends_with(')');
    let inner = text.trim_matches(['[', ']', '(', ')']);
    let Some((low, high)) = inner.split_once(',') else {
        return Some(vec![Cmp {
            op: Op::Eq,
            v: Version::parse(inner)?,
        }]);
    };
    let mut cmps = Vec::new();
    if !low.trim().is_empty() {
        cmps.push(Cmp {
            op: if inclusive_low { Op::Ge } else { Op::Gt },
            v: Version::parse(low)?,
        });
    }
    if !high.trim().is_empty() {
        cmps.push(Cmp {
            op: if inclusive_high { Op::Le } else { Op::Lt },
            v: Version::parse(high)?,
        });
    }
    Some(cmps)
}

/// How far a candidate moves from the current version.
pub(crate) fn level(from: Option<&Version>, to: &Version) -> &'static str {
    let Some(from) = from else { return "latest" };
    match (0..3).find(|i| from.num(*i) != to.num(*i)) {
        Some(0) => "major",
        Some(1) => "minor",
        Some(_) => "patch",
        None => "build",
    }
}

/// What a dependency can move to.
#[derive(Clone, Debug)]
pub(crate) struct Plan {
    pub(crate) latest: Version,
    /// The requirement already allows the latest version.
    pub(crate) satisfied: bool,
    /// The newest version the requirement allows, or `None` when it allows none.
    pub(crate) resolved: Option<Version>,
    /// Newer versions worth offering, oldest first: newest patch, newest minor, latest.
    pub(crate) picks: Vec<Version>,
}

/// Plans updates for `req` from the versions a registry lists. `tagged` is the registry's own
/// `latest` marker (npm dist-tags), which caps stable updates.
pub(crate) fn plan(
    req: &Req,
    versions: &[Version],
    tagged: Option<&Version>,
    prereleases: bool,
) -> Option<Plan> {
    let floor = req.floor();
    let variant = floor.map_or_default(Version::variant);
    let pre = prereleases || floor.is_some_and(Version::is_prerelease);
    let pool: Vec<&Version> = versions
        .iter()
        .filter(|v| {
            v.variant() == variant
                && (pre || (!v.is_prerelease() && tagged.is_none_or(|t| *v <= t)))
        })
        .collect();
    let latest = (*pool.iter().max()?).clone();
    let mut picks: Vec<Version> = Vec::new();
    if let Some(floor) = floor {
        for depth in [2, 1] {
            let same = |v: &&&Version| {
                v.raw != floor.raw && **v > floor && (0..depth).all(|i| v.num(i) == floor.num(i))
            };
            if let Some(v) = pool.iter().filter(same).max() {
                picks.push((*v).clone());
            }
        }
    }
    if floor.is_none_or(|f| latest > *f) {
        picks.push(latest.clone());
    }
    picks.dedup_by(|a, b| a.raw == b.raw);
    Some(Plan {
        satisfied: req.matches(&latest),
        resolved: pool
            .iter()
            .filter(|v| req.matches(v))
            .max()
            .map(|v| (*v).clone()),
        latest,
        picks,
    })
}

#[cfg(test)]
mod tests;
