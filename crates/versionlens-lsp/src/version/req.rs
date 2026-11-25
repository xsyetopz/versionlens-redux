//! Requirement ranges in each registry's syntax: comparators, alternatives, and Maven's brackets.

use super::{Bare, Version};

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
    // A trailing `+` is open-ended: Gradle's `1.+` is any 1.x
    // (docs.gradle.org/current/userguide/dependency_versions.html), and Raku's `1.0.1+` allows
    // larger versions too (docs.raku.org/type/Version#method_plus).
    if let Some(head) = text.strip_suffix('+').filter(|_| op.is_none()) {
        let prefix = head.ends_with('.');
        return Some(Cmp {
            op: if prefix { Op::Prefix } else { Op::Ge },
            v: Version::parse(head.trim_end_matches('.'))?,
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
