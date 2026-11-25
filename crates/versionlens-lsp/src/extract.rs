//! Turns a manifest into dependencies with the rules its file type lists.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{LazyLock, Mutex, PoisonError};

use regex_automata::meta::Regex;
use regex_automata::util::captures::Captures;

use crate::catalog::FileType;
use crate::flatten::{Leaf, flatten};

/// One way to find dependencies in a manifest. Paths are dotted; `*` matches one segment.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Rule {
    /// A map of `name = requirement`, where the value may also sit under `name.version`.
    Keys(&'static str),
    /// Strings such as `name>=1.0` at a path, read with a regex that names `name` and `version`.
    Spec(&'static str, &'static str),
    /// Objects at a path whose fields name the dependency: `groupId:artifactId` joins two fields,
    /// `@Include|@Update` takes the first present. The last field names an optional source URL.
    Record(&'static str, &'static str, &'static str, &'static str),
    /// A regex over the whole text with `name`, optional `group`, `version`, `registry`, and `source` groups.
    /// Without a `name` group the file type's id names the package, as in `.nvmrc` for `node`.
    Lines(&'static str),
    /// A rule whose dependencies come from another registry, such as the `pip:` list inside a
    /// conda `environment.yml`.
    Via(&'static str, &'static Self),
}

/// A dependency found in a manifest: what it is called, what it asks for, and where that is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Dep {
    pub(crate) name: String,
    pub(crate) req: String,
    /// Byte span of the requirement; empty where a bare name gets a version inserted.
    pub(crate) span: Range<usize>,
    pub(crate) registry: &'static str,
    pub(crate) source: Option<String>,
}

/// Compiles each rule's regex once for the life of the process.
pub(crate) fn regex(pattern: &'static str) -> Regex {
    static CACHE: LazyLock<Mutex<HashMap<&'static str, Regex>>> = LazyLock::new(Mutex::default);
    let mut cache = CACHE.lock().unwrap_or_else(PoisonError::into_inner);
    cache
        .entry(pattern)
        .or_insert_with(|| {
            Regex::new(pattern)
                .unwrap_or_else(|_| Regex::new(r"[^\s\S]").unwrap_or_else(|_| unreachable!()))
        })
        .clone()
}

fn path_matches(pattern: &str, path: &[String]) -> bool {
    let mut parts = pattern.split('.');
    path.iter()
        .all(|seg| parts.next().is_some_and(|p| p == "*" || p == seg))
        && parts.next().is_none()
}

/// The registry id of dependencies on a local directory or file, which need no lookup.
pub(crate) const DIRECTORY: &str = "directory";

/// The local path a requirement names, such as npm's `file:../a` or a `../a` path, if it names one.
pub(crate) fn local_path(req: &str) -> Option<&str> {
    const SCHEMES: &[&str] = &["file://", "file:", "link:", "portal:"];
    SCHEMES
        .iter()
        .find_map(|s| req.strip_prefix(s))
        .or_else(|| {
            ["./", "../", "/", "~/"]
                .iter()
                .any(|p| req.starts_with(p))
                .then_some(req)
        })
        .filter(|path| !path.is_empty() && !path.starts_with("//"))
}

/// Requirements a registry cannot answer: paths, URLs, git refs, workspace links, and templating.
fn is_registry_req(req: &str) -> bool {
    const LOCAL: &[&str] = &[
        "/",
        "./",
        "../",
        "~/",
        "file:",
        "link:",
        "workspace:",
        "catalog:",
        "portal:",
        "path:",
        "git",
        "github:",
        "gitlab:",
        "bitbucket:",
        "http:",
        "https:",
        "npm:",
        "jsr:",
        "sdk:",
        "patch:",
        "exec:",
    ];
    !(req.contains("://")
        || req.contains('$')
        || req.contains("{{")
        || LOCAL.iter().any(|p| req.starts_with(p)))
}

/// Names that are platforms or toolchains rather than packages. The check is per registry because
/// `python` is a runtime in a `PyPI` requirement but the package itself in `.python-version`.
fn is_package(name: &str, registry: &str) -> bool {
    !(matches!(
        (registry, name),
        ("composer", "php")
            | ("pypi", "python")
            | ("julia", "julia")
            | ("cran", "R")
            | ("luarocks", "lua")
    ) || name.contains('$')
        || (registry == "composer"
            && (name.starts_with("ext-")
                || name.starts_with("lib-")
                || name.starts_with("composer-"))))
}

pub(crate) fn extract(file: &FileType, text: &str) -> Vec<Dep> {
    let leaves = flatten(file.format, text);
    let mut deps = Vec::new();
    for rule in file.rules {
        match *rule {
            Rule::Keys(map) => keys(map, &leaves, file.registry, &mut deps),
            Rule::Spec(path, pattern) => {
                let re = regex(pattern);
                for leaf in leaves
                    .iter()
                    .filter(|l| path_matches(path, &l.path) && text[l.span.clone()] == l.value)
                {
                    if let Some(caps) = re.captures_iter(&leaf.value).next() {
                        deps.extend(from_caps(&caps, &leaf.value, leaf.span.start, file));
                    }
                }
            }
            Rule::Record(path, names, version, source) => {
                records(path, (names, version, source), &leaves, file, &mut deps);
            }
            Rule::Lines(pattern) => deps.extend(
                regex(pattern)
                    .captures_iter(text)
                    .filter_map(|c| from_caps(&c, text, 0, file)),
            ),
            Rule::Via(registry, rule) => {
                let inner = FileType {
                    registry,
                    rules: std::slice::from_ref(rule),
                    ..*file
                };
                deps.extend(extract(&inner, text));
            }
        }
    }
    for dep in &mut deps {
        if local_path(&dep.req).is_some() {
            dep.registry = DIRECTORY;
        }
        if let Some(var) = dep.req.strip_prefix("${").and_then(|r| r.strip_suffix('}'))
            && let Some(leaf) = leaves
                .iter()
                .find(|l| l.path.last().is_some_and(|k| k == var))
        {
            dep.req.clone_from(&leaf.value);
            dep.span = leaf.span.clone();
        }
    }
    deps.retain(|d| {
        !d.name.is_empty()
            && is_package(&d.name, d.registry)
            && (d.registry == DIRECTORY || is_registry_req(&d.req))
    });
    deps.sort_by_key(|d| (d.span.start, d.span.end));
    deps.dedup_by_key(|d| d.span.clone());
    deps
}

fn keys(map: &str, leaves: &[Leaf], registry: &'static str, deps: &mut Vec<Dep>) {
    let depth = map.split('.').count();
    for leaf in leaves {
        let (Some(head), Some(name)) = (leaf.path.get(..depth), leaf.path.get(depth)) else {
            continue;
        };
        let tail = &leaf.path[depth + 1..];
        // A `path` entry, as Cargo and Pub write local dependencies, names a directory.
        let local = tail == ["path"];
        if path_matches(map, head) && (tail.is_empty() || tail == ["version"] || local) {
            deps.push(Dep {
                name: name.clone(),
                req: leaf.value.clone(),
                span: leaf.span.clone(),
                registry: if local { DIRECTORY } else { registry },
                source: None,
            });
        }
    }
}

fn records(
    path: &str,
    fields: (&str, &str, &str),
    leaves: &[Leaf],
    file: &FileType,
    deps: &mut Vec<Dep>,
) {
    let (names, version, source) = fields;
    let depth = path.split('.').count();
    let mut groups: Vec<(usize, Vec<&Leaf>)> = Vec::new();
    for leaf in leaves
        .iter()
        .filter(|l| l.path.len() == depth + 1 && path_matches(path, &l.path[..depth]))
    {
        match groups.iter_mut().find(|(id, _)| *id == leaf.parent) {
            Some((_, group)) => group.push(leaf),
            None => groups.push((leaf.parent, vec![leaf])),
        }
    }
    for (_, group) in groups {
        let field = |spec: &str| {
            spec.split('|').find_map(|key| {
                group
                    .iter()
                    .find(|l| l.path.last().is_some_and(|k| k == key))
                    .copied()
            })
        };
        let parts: Option<Vec<&str>> = names
            .split(':')
            .map(|n| field(n).map(|l| l.value.as_str()))
            .collect();
        let (Some(parts), Some(leaf)) = (parts, field(version)) else {
            continue;
        };
        deps.push(Dep {
            name: parts.join(":"),
            req: leaf.value.clone(),
            span: leaf.span.clone(),
            registry: file.registry,
            source: field(source).map(|l| l.value.clone()),
        });
    }
}

fn from_caps(caps: &Captures, text: &str, offset: usize, file: &FileType) -> Option<Dep> {
    let group = |name: &str| caps.get_group_by_name(name).map(|s| &text[s.range()]);
    let version = caps.get_group_by_name("version").map(|s| s.range());
    let (name, span) = match (caps.get_group_by_name("name"), group("group")) {
        (Some(n), Some(g)) => (
            format!("{g}:{}", &text[n.range()]),
            version.unwrap_or(n.end..n.end),
        ),
        (Some(n), None) => (text[n.range()].to_owned(), version.unwrap_or(n.end..n.end)),
        (None, _) => (file.id.to_owned(), version?),
    };
    let registry = group("registry")
        .and_then(crate::registry::alias)
        .unwrap_or(file.registry);
    Some(Dep {
        name,
        req: text[span.clone()].to_owned(),
        span: span.start + offset..span.end + offset,
        registry,
        source: group("source").map(str::to_owned),
    })
}

#[cfg(test)]
mod tests;
