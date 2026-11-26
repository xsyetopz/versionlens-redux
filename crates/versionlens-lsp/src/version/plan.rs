//! What a dependency can move to from its requirement and the versions a registry lists.

use super::req::Req;
use super::{Version, split};

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
    /// The original 1.x suggestions, newest first, one per version: each labelled `latest`,
    /// `major`, `minor`, `patch`, or `bump`.
    pub(crate) picks: Vec<(&'static str, Version)>,
    /// What the bulk updates move the requirement's floor to, oldest first: its newest patch,
    /// its newest minor, and the latest.
    pub(crate) steps: Vec<Version>,
    /// The other builds of the resolved version, such as `20.1.0-bookworm` beside `20.1.0-alpine`.
    pub(crate) builds: Vec<Version>,
    /// The yanked version the requirement resolves to when no listed version matches.
    pub(crate) yanked: Option<Version>,
    /// The registry marks the resolved version deprecated.
    pub(crate) deprecated: bool,
}

impl Plan {
    /// Applies the registry's yanked and deprecated marks: as the original 1.x did, a requirement
    /// that only yanked versions satisfy resolves to the newest of them.
    pub(crate) fn mark(&mut self, req: &Req, yanked: &[Version], deprecated: &[Version]) {
        if self.resolved.is_none() {
            self.yanked = yanked.iter().filter(|v| req.matches(v)).max().cloned();
        }
        self.deprecated = self
            .resolved
            .as_ref()
            .is_some_and(|r| deprecated.iter().any(|d| d.raw == r.raw));
    }
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
    // A registry whose own marker is a prerelease, as npm's `canary` dist-tag that a
    // `.bun-version` of `canary` follows, offers prereleases up to it.
    let tagged_pre = tagged.is_some_and(Version::is_prerelease);
    let pool: Vec<&Version> = versions
        .iter()
        .filter(|v| {
            v.variant() == variant
                && (pre || ((tagged_pre || !v.is_prerelease()) && tagged.is_none_or(|t| *v <= t)))
        })
        .collect();
    let latest = *pool.iter().max()?;
    let resolved = pool.iter().filter(|v| req.matches(v)).max().copied();
    Some(Plan {
        satisfied: req.matches(latest),
        picks: suggest(&pool, latest, resolved, floor),
        steps: steps(&pool, latest, floor),
        builds: resolved.map_or_default(|r| builds(versions, r)),
        yanked: None,
        deprecated: false,
        resolved: resolved.cloned(),
        latest: latest.clone(),
    })
}

/// The listed releases with the numbers of `resolved` but another suffix: the builds a Docker tag
/// or a `+build` version can switch between.
fn builds(versions: &[Version], resolved: &Version) -> Vec<Version> {
    let suffix = split(&resolved.raw).2;
    let len = resolved.nums.len();
    versions
        .iter()
        .filter(|v| {
            !v.is_prerelease()
                && (0..len.max(v.nums.len())).all(|i| v.num(i) == resolved.num(i))
                && split(&v.raw).2 != suffix
        })
        .cloned()
        .collect()
}

fn steps(pool: &[&Version], latest: &Version, floor: Option<&Version>) -> Vec<Version> {
    let mut out: Vec<Version> = Vec::new();
    if let Some(floor) = floor {
        for depth in [2, 1] {
            let same = |v: &&&Version| {
                v.raw != floor.raw && **v > floor && (0..depth).all(|i| v.num(i) == floor.num(i))
            };
            if let Some(v) = pool.iter().filter(same).max() {
                out.push((*v).clone());
            }
        }
    }
    if floor.is_none_or(|f| latest > f) {
        out.push(latest.clone());
    }
    out.dedup_by(|a, b| a.raw == b.raw);
    out
}

/// The original's release suggestions: the latest version unless the requirement already sits
/// exactly on it, the newest of the next major line and of the minor and patch lines past the
/// version in use, and a bump of a range up to the newest version it allows.
fn suggest(
    pool: &[&Version],
    latest: &Version,
    resolved: Option<&Version>,
    floor: Option<&Version>,
) -> Vec<(&'static str, Version)> {
    let is_latest = resolved == Some(latest);
    // The range admits versions older than the one it resolves to, so it can be raised.
    let range_update = floor.is_none_or(|f| resolved.is_some_and(|r| r > f));
    let mut found: Vec<(&'static str, &Version)> = Vec::new();
    if !is_latest || range_update {
        found.push(("latest", latest));
    }
    if let Some(base) = resolved.or(floor) {
        let newest =
            |test: &dyn Fn(&Version) -> bool| pool.iter().copied().filter(|v| test(v)).max();
        let next = pool
            .iter()
            .map(|v| v.num(0))
            .filter(|m| *m > base.num(0))
            .min();
        let same = |v: &Version, n: usize| (0..n).all(|i| v.num(i) == base.num(i));
        let lines = [
            ("major", newest(&|v| Some(v.num(0)) == next)),
            ("minor", newest(&|v| same(v, 1) && v.num(1) > base.num(1))),
            ("patch", newest(&|v| same(v, 2) && v.num(2) > base.num(2))),
        ];
        found.extend(lines.into_iter().filter_map(|(label, v)| Some((label, v?))));
    }
    if let Some(r) = resolved.filter(|_| !is_latest && range_update) {
        found.push(("bump", r));
    }
    let mut picks: Vec<(&'static str, Version)> = Vec::new();
    for (label, v) in found {
        if !picks.iter().any(|(_, p)| p.raw == v.raw) {
            picks.push((label, v.clone()));
        }
    }
    picks.sort_by(|a, b| b.1.cmp(&a.1));
    picks
}

/// The next releases of a manifest's own version, as upstream's `getProjectVersionSuggestions`
/// offers them through semver's `inc`: `major`, `minor`, and `patch` for a release, and for a
/// prerelease its `release` and next `prerelease`. A version that does not parse counts as
/// `0.0.0`, and missing segments count as `0`; the `v` prefix stays.
pub(crate) fn increments(own: &str) -> Vec<(&'static str, String)> {
    let own = own.trim();
    let (prefix, _, suffix) = split(own);
    let nums = Version::parse(own).map_or_else(Vec::new, |v| v.nums);
    let [major, minor, patch] = [0, 1, 2].map(|i| nums.get(i).copied().unwrap_or(0));
    let release = |m: u64, n: u64, p: u64| format!("{prefix}{m}.{n}.{p}");
    let pre = suffix.split('+').next().unwrap_or_default();
    let Some(pre) = pre.strip_prefix('-').filter(|p| !p.is_empty()) else {
        return vec![
            ("major", release(major + 1, 0, 0)),
            ("minor", release(major, minor + 1, 0)),
            ("patch", release(major, minor, patch + 1)),
        ];
    };
    // semver bumps the last numeric identifier, or appends `.0` when there is none.
    let next = match pre.rsplit_once('.') {
        Some((head, last)) if last.parse::<u64>().is_ok() => {
            format!("{head}.{}", last.parse::<u64>().unwrap_or(0) + 1)
        }
        _ if pre.parse::<u64>().is_ok() => (pre.parse::<u64>().unwrap_or(0) + 1).to_string(),
        _ => format!("{pre}.0"),
    };
    let current = release(major, minor, patch);
    vec![
        ("release", current.clone()),
        ("prerelease", format!("{current}-{next}")),
    ]
}
