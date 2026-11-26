//! Dependency maps, `name = requirement`, with the long forms Cargo, Pub, uv, dub, and npm's
//! overrides write under a name.

use super::{DIRECTORY, Dep, SOURCE, path_matches, source_label};
use crate::flatten::Leaf;

pub(super) fn keys(map: &str, leaves: &[Leaf], registry: &'static str, deps: &mut Vec<Dep>) {
    let depth = map.split('.').count();
    for leaf in leaves {
        let (Some(head), Some(name)) = (leaf.path.get(..depth), leaf.path.get(depth)) else {
            continue;
        };
        let tail = &leaf.path[depth + 1..];
        // A list in the map's place, as devbox.json also writes `packages`, flattens to index
        // names; its entries are read by a `spec` rule instead.
        if !path_matches(map, head) || name.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        // A `path` entry, as Cargo and Pub write local dependencies, names a directory. A `git`
        // entry, as Cargo, Pub (`git.url` in its long form), and uv write them, names a
        // repository, and uv's `url` an archive.
        // Pub's `sdk: flutter` comes with the SDK, as pub.dev's `flutter` is a 2016 placeholder,
        // and a `hosted` package with no `version` beside it takes any version:
        // dart.dev/tools/pub/dependencies#sdk and #hosted-packages
        // dub's `repository` names a Git repository whose `version` beside it is a tag or commit,
        // not a registry version: dub.pm/dub-reference/dependencies/#git-repositories
        // pnpm's `-` value removes the dependency the key selects (pnpm.io/settings#overrides).
        if registry == "npm" && leaf.value == "-" {
            continue;
        }
        let mut req = leaf.value.clone();
        let mut span = leaf.span.clone();
        let (registry, source) = match tail.iter().map(String::as_str).collect::<Vec<_>>()[..] {
            ["version"] if sibling(leaves, leaf, depth, "repository") => continue,
            [] | ["version"] => (registry, None),
            ["path"] => (DIRECTORY, None),
            ["git"] | ["git", "url"] => (SOURCE, Some("git repository")),
            ["url" | "repository"] if source_label(&leaf.value).is_some() => {
                (SOURCE, source_label(&leaf.value))
            }
            ["sdk"] => (SOURCE, Some("SDK package")),
            ["hosted"] | ["hosted", "url"] if !sibling(leaves, leaf, depth, "version") => {
                req.clear();
                span = span.start..span.start;
                (registry, None)
            }
            _ => continue,
        };
        let name = if registry == "npm" {
            selected(name, head.last().map(String::as_str))
        } else {
            name.clone()
        };
        deps.push(Dep {
            name,
            req,
            span,
            registry,
            source: source.map(str::to_owned),
            index: index(leaf, leaves),
            ..Dep::default()
        });
    }
}

/// Whether the dependency `leaf` sits under, at `depth` in its path, has a `key` entry of its own.
fn sibling(leaves: &[Leaf], leaf: &Leaf, depth: usize, key: &str) -> bool {
    leaves.iter().any(|l| {
        l.path.len() == depth + 2
            && l.path.starts_with(&leaf.path[..=depth])
            && l.path[depth + 1] == key
    })
}

/// The package an npm-family override key selects, which a registry knows by name alone. npm's
/// `"."` sets the package whose table holds it, and a key may name a version, as in `axios@<1`
/// (docs.npmjs.com/cli/v10/configuring-npm/package-json#overrides), the parent it applies below,
/// as in pnpm's `foo@1>bar` (<https://pnpm.io/package_json#pnpmoverrides>), or a path to the package, as in
/// Yarn's `@babel/core/**/json5` (classic.yarnpkg.com/lang/en/docs/selective-version-resolutions/).
/// A dependency map's keys are bare names, which this leaves as they are.
fn selected(key: &str, parent: Option<&str>) -> String {
    if key == "." {
        return parent.map_or_else(String::new, |p| selected(p, None));
    }
    // A `>` before a letter or scope starts the child; one before a digit or `=` opens a range,
    // as in `@volar/typescript>typescript@>=7`.
    let child = key
        .rmatch_indices('>')
        .find(|(at, _)| {
            key[at + 1..]
                .bytes()
                .next()
                .is_some_and(|b| b.is_ascii_alphabetic() || b == b'@' || b == b'_')
        })
        .map_or(0, |(at, _)| at + 1);
    let key = &key[child..];
    let mut parts = key.rsplit('/');
    let last = parts.next().unwrap_or(key);
    let package = match parts.next() {
        Some(scope) if scope.starts_with('@') => &key[key.len() - last.len() - scope.len() - 1..],
        _ => last,
    };
    // The version follows the first `@` after a scope's own.
    let end = package
        .get(1..)
        .and_then(|rest| rest.find('@'))
        .map_or(package.len(), |at| at + 1);
    package[..end].to_owned()
}

/// The registry a `version` leaf's table names beside it: Cargo's `registry`, Poetry's `source`,
/// or Pipenv's `index`.
fn index(leaf: &Leaf, leaves: &[Leaf]) -> Option<String> {
    let (table, last) = leaf.path.split_at(leaf.path.len().checked_sub(1)?);
    (last == ["version"]).then_some(())?;
    leaves
        .iter()
        .find(|l| {
            l.parent == leaf.parent
                && l.path.split_last().is_some_and(|(key, head)| {
                    head == table && ["registry", "source", "index"].contains(&key.as_str())
                })
        })
        .map(|l| l.value.clone())
}
