//! Turns a manifest into dependencies with the rules its file type lists.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::{LazyLock, Mutex, PoisonError};

use regex_automata::meta::Regex;
use regex_automata::util::captures::Captures;

use crate::catalog::FileType;
use crate::flatten::{Format, Leaf, flatten};

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
    /// A regex over the whole text with `name`, optional `group`, `version`, `registry`, and `source`
    /// groups, or a `ref` group in place of `version` that names the key holding the version.
    /// Without a `name` group the file type's id names the package, as in `.nvmrc` for `node`.
    Lines(&'static str),
    /// A rule whose dependencies come from another registry, such as the `pip:` list inside a
    /// conda `environment.yml`.
    Via(&'static str, &'static Self),
    /// A rule whose dependency names lack a fixed head or tail the manifest leaves out, such as
    /// `org.jetbrains.kotlin.` before the `jvm` of Gradle's `kotlin("jvm")`, or `_2.13` after the
    /// artifact of sbt's `%%`, which the build's Scala version gives.
    Affix(&'static str, &'static str, &'static Self),
    /// A rule read only inside each match of a regex, or of its `body` group, else its first
    /// group, where it has one, because a bare name means a dependency only in its list: `"lwt"`
    /// in an opam `depends` list, not in `tags`. Regexes have no lookbehind, so a lines rule
    /// alone cannot tell them apart. A block's `source` group is the source of each dependency
    /// in it that names none, as a Gemfile's `path "../p" do` block is for the gems inside.
    Within(&'static str, &'static Self),
}

/// A dependency found in a manifest: what it is called, what it asks for, and where that is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Dep {
    pub(crate) name: String,
    pub(crate) req: String,
    /// Byte span of the requirement; empty where a bare name gets a version inserted.
    pub(crate) span: Range<usize>,
    pub(crate) registry: &'static str,
    pub(crate) source: Option<String>,
    /// The registry or index the manifest names for it, such as Cargo's `registry = "name"`.
    pub(crate) index: Option<String>,
    /// Where the project's registry configuration sends its lookup; empty for the default.
    pub(crate) targets: Vec<crate::project::Target>,
    /// The version is the one installed, from a `pinned` file type such as a lockfile.
    pub(crate) pinned: bool,
    /// An update writes the whole version, from a `precise` file type such as Carthage's.
    pub(crate) precise: bool,
    /// The manifest writes a lower-bound operator outside the span, which a rule captures as
    /// `min`, as opam's `{>= "1.0"}` keeps it outside the quoted version.
    pub(crate) min: bool,
}

impl Dep {
    /// What a version without an operator means for this dependency: the exact one where the
    /// file pins it, at least it where the manifest says so outside the span, else what it means
    /// to the registry.
    pub(crate) fn bare(&self, registry: &crate::registry::Registry) -> crate::version::Bare {
        if self.pinned {
            crate::version::Bare::Exact
        } else if self.min {
            crate::version::Bare::Min
        } else {
            registry.bare
        }
    }
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

/// Whether a leaf's `path` fits a dotted pattern, where `*` stands for any key or list index and
/// `#` for a list index alone, as npm reads `bundleDependencies` as a list of names but an object
/// by its keys.
fn path_matches(pattern: &str, path: &[String]) -> bool {
    let mut parts = pattern.split('.');
    path.iter().all(|seg| {
        parts.next().is_some_and(|p| {
            p == "*"
                || p == seg
                || p == "#" && !seg.is_empty() && seg.bytes().all(|b| b.is_ascii_digit())
        })
    }) && parts.next().is_none()
}

/// The dependencies `rule` finds, their names completed by `prefix` and `suffix`. An affix
/// completes a registry's package name; a package its requirement turns local or git keeps the
/// name the manifest gives it. A rule routed to a source on purpose, such as qlfile's
/// `github owner/repo :branch`, affixes the URL its link needs.
fn affix(file: &FileType, prefix: &str, suffix: &str, rule: &'static Rule, text: &str) -> Vec<Dep> {
    let inner = FileType {
        rules: std::slice::from_ref(rule),
        ..*file
    };
    let mut deps = read(&inner, text);
    for dep in &mut deps {
        if dep.registry == file.registry || ![DIRECTORY, SOURCE].contains(&dep.registry) {
            dep.name.insert_str(0, prefix);
            dep.name.push_str(suffix);
        }
    }
    deps
}

pub(crate) fn extract(file: &FileType, text: &str) -> Vec<Dep> {
    let mut deps = read(file, text);
    // West projects join the manifest's remotes and defaults, which no single rule's match
    // holds. Those that inherit `defaults.revision` share its span, so they join after the
    // dedup of values two rules read.
    if file.id == "west" {
        let leaves = flatten(file.format, text);
        let mut projects = west::projects(&leaves);
        for dep in &mut projects {
            settle(dep, file.pinned, &leaves);
        }
        projects.retain(|p| !deps.iter().any(|d| d.span == p.span && d.name == p.name));
        deps.extend(projects);
        deps.sort_by_key(|d| (d.span.start, d.span.end));
    }
    deps
}

/// The dependencies of `deps` under the dotted `paths` an original `dependencyProperties` setting
/// lists, as upstream `VersionLens` 1.x read a manifest only there: a dependency stays when the
/// leaf that holds it sits under one of them. A path under which no rule found one adds the keys
/// of the object there, as upstream's JSON, TOML, and YAML parsers read a listed object's keys
/// (`src/domain/parsers` of upstream 1.3.0); an XML path only narrows, since an element's
/// children are not dependency names. A line-read manifest has no paths, so it keeps them all.
pub(crate) fn properties(
    file: &FileType,
    text: &str,
    deps: Vec<Dep>,
    paths: &[String],
) -> Vec<Dep> {
    if file.format == Format::Text || file.format == Format::Ini {
        return deps;
    }
    let leaves = flatten(file.format, text);
    let under = |path: &[String], pattern: &str| {
        let depth = pattern.split('.').count();
        path.get(..depth)
            .is_some_and(|head| path_matches(pattern, head))
    };
    // The leaf holding a dependency spans it, or, for a bare name's empty span, starts last
    // before it.
    let holder = |dep: &Dep| {
        leaves
            .iter()
            .rev()
            .find(|l| l.span.start <= dep.span.start && dep.span.end <= l.span.end)
            .or_else(|| {
                leaves
                    .iter()
                    .filter(|l| l.span.start <= dep.span.start)
                    .max_by_key(|l| l.span.start)
            })
    };
    let (mut kept, dropped): (Vec<Dep>, Vec<Dep>) = deps
        .into_iter()
        .partition(|d| holder(d).is_some_and(|l| paths.iter().any(|p| under(&l.path, p))));
    drop(dropped);
    if file.format == Format::Xml {
        return kept;
    }
    for path in paths {
        if kept
            .iter()
            .any(|d| holder(d).is_some_and(|l| under(&l.path, path)))
        {
            continue;
        }
        let mut found = Vec::new();
        keys(path, &leaves, file.registry, &mut found);
        for mut dep in found {
            settle(&mut dep, file.pinned, &leaves);
            kept.push(dep);
        }
    }
    kept.sort_by_key(|d| (d.span.start, d.span.end));
    kept.dedup_by(|a, b| a.span == b.span);
    kept
}

/// The dependencies `rule` finds inside each block `block` matches, shifted to the file's
/// offsets.
fn within(file: &FileType, block: &'static str, rule: &'static Rule, text: &str) -> Vec<Dep> {
    let mut found = Vec::new();
    let inner = FileType {
        rules: std::slice::from_ref(rule),
        ..*file
    };
    for caps in regex(block).captures_iter(text) {
        // A block that captures a `suffix`, such as the full Scala version Mill's
        // `:::` appends, holds its rule in the whole match.
        let suffix = caps.get_group_by_name("suffix").map(|m| &text[m.range()]);
        let source = caps.get_group_by_name("source").map(|m| &text[m.range()]);
        let first = caps
            .get_group_by_name("body")
            .or_else(|| caps.get_group(1).filter(|_| suffix.is_none()));
        let Some(range) = first.or_else(|| caps.get_group(0)) else {
            continue;
        };
        let range = range.range();
        found.extend(
            read(&inner, &text[range.clone()])
                .into_iter()
                .map(|mut dep| {
                    dep.span = dep.span.start + range.start..dep.span.end + range.start;
                    if dep.source.is_none() {
                        dep.source = source.map(str::to_owned);
                    }
                    if let Some(suffix) = suffix
                        && ![DIRECTORY, SOURCE].contains(&dep.registry)
                    {
                        dep.name.push_str(suffix);
                    }
                    dep
                }),
        );
    }
    found
}

/// The dependencies the file type's rules find, settled and deduplicated.
fn read(file: &FileType, text: &str) -> Vec<Dep> {
    let leaves = flatten(file.format, text);
    let mut deps = Vec::new();
    // A version a reference resolves, as a POM's `${spring.version}` or a Gradle catalog's
    // `version.ref`, lands on the leaf it names, which several packages may share. Each keeps its
    // own span through the deduplication below, which would otherwise keep one of them, and moves
    // after it.
    let mut moves: Vec<(Range<usize>, Range<usize>)> = Vec::new();
    for rule in file.rules {
        match *rule {
            Rule::Keys(map) => keys(map, &leaves, file.registry, &mut deps),
            Rule::Spec(path, pattern) => {
                let re = regex(pattern);
                for leaf in leaves.iter().filter(|l| path_matches(path, &l.path)) {
                    let raw = &text[leaf.span.clone()];
                    let Some(at) = unescaped(raw, &leaf.value) else {
                        continue;
                    };
                    // A value may list several, as library.properties' `depends=A, B (>=1.0)`.
                    for caps in re.captures_iter(&leaf.value) {
                        deps.extend(from_caps(&caps, &leaf.value, 0, file).map(|mut dep| {
                            dep.span = leaf.span.start + at[dep.span.start]
                                ..leaf.span.start + at[dep.span.end];
                            dep
                        }));
                    }
                }
            }
            Rule::Record(path, names, version, source) => {
                records(
                    path,
                    (names, version, source),
                    &leaves,
                    file,
                    &mut deps,
                    &mut moves,
                );
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
                deps.extend(read(&inner, text));
            }
            Rule::Affix(prefix, suffix, rule) => {
                deps.extend(affix(file, prefix, suffix, rule, text));
            }
            Rule::Within(block, rule) => deps.extend(within(file, block, rule, text)),
        }
    }
    for dep in &mut deps {
        if let Some(target) = settle(dep, file.pinned, &leaves) {
            moves.push((dep.span.clone(), target));
        }
        dep.precise |= file.precise;
    }
    // An untagged `FROM build` may name an earlier stage, `FROM golang:1.22 AS build`, and no
    // image: docs.docker.com/build/building/multi-stage/. A regex cannot refer back to a stage
    // name, so the names are gathered here.
    let stages: Vec<&str> = regex(r"(?mi)^[ \t]*FROM[ \t][^\n]*?[ \t]AS[ \t]+([\w.-]+)")
        .captures_iter(text)
        .filter_map(|c| c.get_group(1).map(|s| &text[s.range()]))
        .collect();
    deps.retain(|d| {
        !(d.registry == "docker"
            && d.req.is_empty()
            && stages.iter().any(|s| s.eq_ignore_ascii_case(&d.name)))
    });
    // A Julia `[deps]` entry gives way to the `[compat]` bound of the same package.
    let bounded: Vec<String> = deps
        .iter()
        .filter(|d| d.registry == "julia")
        .map(|d| d.name.clone())
        .collect();
    deps.retain(|d| {
        !(d.registry == "julia-stdlib" && d.req.is_empty() && bounded.contains(&d.name))
    });
    deps.retain(|d| {
        !d.name.is_empty()
            && is_package(&d.name, d.registry)
            && ([DIRECTORY, SOURCE].contains(&d.registry) || is_registry_req(&d.req))
    });
    // A value two rules read keeps the narrower reading a registry answers: JSPM's
    // `github:owner/repo@1.0.0` is a git source to its map's `keys` rule and a GitHub release,
    // spanning `1.0.0`, to its `spec` rule.
    // A git ref reading, such as GitLab's `ref: refs/tags/v1.0.0`, gives way to the tag inside it.
    let loose = |d: &Dep| [SOURCE, crate::git_ref::GIT_REF].contains(&d.registry);
    let looked_up: Vec<Range<usize>> = deps
        .iter()
        .filter(|d| !loose(d) && !d.span.is_empty())
        .map(|d| d.span.clone())
        .collect();
    deps.retain(|d| {
        !loose(d)
            || !looked_up
                .iter()
                .any(|s| *s != d.span && d.span.start <= s.start && s.end <= d.span.end)
    });
    deps.sort_by_key(|d| (d.span.start, d.span.end));
    // A manifest's own version keeps its local increments beside a lookup of the published
    // package, as `jsr.json` gets both upstream's increments and the fork's master's JSR lookup.
    deps.dedup_by(|a, b| a.span == b.span && (a.registry == PROJECT) == (b.registry == PROJECT));
    for dep in &mut deps {
        if let Some((_, target)) = moves.iter().find(|(own, _)| *own == dep.span) {
            dep.span = target.clone();
        }
    }
    deps.sort_by_key(|d| (d.span.start, d.span.end));
    deps
}

/// Settles where a dependency resolves: a local or git source, a registry, or the key a
/// `${ref}` names, whose span it returns for the dependency to move to.
fn settle(dep: &mut Dep, pinned: bool, leaves: &[Leaf]) -> Option<Range<usize>> {
    dep.pinned |= pinned;
    // A Helm chart's `repository` may be a local chart, a `helm repo add` alias, or an OCI
    // registry: helm.sh/docs/helm/helm_dependency/ and helm.sh/docs/topics/registries/
    match dep.source.as_deref() {
        // A bare name in a block that gives the path, as a Gemfile's `path` block does, keeps
        // the path as its source, which its lens links to.
        Some(source) if local_path(source).is_some() || dep.registry == DIRECTORY => {
            dep.registry = DIRECTORY;
        }
        // A Cargo member's `workspace = true` takes its version from the workspace root, and a Mix
        // `in_umbrella: true` app is a sibling app of the same umbrella.
        Some("workspace" | "in_umbrella") => {
            dep.registry = SOURCE;
            dep.source = Some("workspace package".to_owned());
        }
        Some(source) if source.starts_with('@') || source.starts_with("alias:") => {
            dep.registry = SOURCE;
            dep.source = Some("repository alias".to_owned());
        }
        Some(source) if let Some(repo) = source.strip_prefix("oci://") => {
            dep.name = format!("{}/{}", repo.trim_end_matches('/'), dep.name);
            dep.registry = "oci";
        }
        // An Ansible role's `src` may name its git repository rather than a Galaxy role:
        // docs.ansible.com/ansible/latest/galaxy/user_guide.html#installing-multiple-roles-from-a-file
        Some(source) if source_label(source) == Some("git repository") => {
            dep.registry = SOURCE;
            dep.source = Some("git repository".to_owned());
        }
        _ => {}
    }
    if dep.registry == DIRECTORY || local_path(&dep.req).is_some() {
        dep.registry = DIRECTORY;
    } else if dep.registry != SOURCE
        && let Some(label) = source_label(&dep.req)
    {
        dep.registry = SOURCE;
        dep.source = Some(label.to_owned());
    }
    // A `via = 'source'` row pins a branch or revision of the repository its name gives.
    if dep.registry == SOURCE && dep.source.is_none() {
        let label = source_label(&dep.name).unwrap_or("git reference");
        dep.source = Some(label.to_owned());
    }
    // A Gradle catalog's version may be a rich one, `foo = { strictly = "1.0" }`, whose
    // `require`, `strictly`, or `prefer` holds it:
    // docs.gradle.org/current/userguide/version_catalogs.html#sec:common-changes
    // An MSBuild property reads `$(Name)`, and a `PropertyGroup` sets it:
    // learn.microsoft.com/visualstudio/msbuild/msbuild-properties
    let var = property(&dep.req);
    if let Some(var) = var
        && let Some(leaf) = leaves.iter().find(|l| match &l.path[..] {
            [.., key] if key == var => true,
            [.., key, kind] => key == var && ["require", "strictly", "prefer"].contains(&&**kind),
            _ => false,
        })
    {
        dep.req.clone_from(&leaf.value);
        return Some(leaf.span.clone());
    }
    // A property this file does not set comes from one it inherits, such as a parent POM
    // (maven.apache.org/pom.html#Inheritance), a `Directory.Build.props`
    // (learn.microsoft.com/visualstudio/msbuild/customize-by-directory), or Gradle's
    // `buildSrc`, as Cargo's `workspace = true` takes its version from the workspace root.
    if var.is_some() && ["nuget", "maven"].contains(&dep.registry) {
        dep.registry = SOURCE;
        dep.source = Some("version property".to_owned());
    }
    None
}

/// The property a requirement consists of: `${name}`, Gradle's `$name`, or `MSBuild`'s `$(Name)`.
fn property(req: &str) -> Option<&str> {
    let var = req
        .strip_prefix("${")
        .and_then(|r| r.strip_suffix('}'))
        .or_else(|| req.strip_prefix("$(").and_then(|r| r.strip_suffix(')')))
        .or_else(|| req.strip_prefix('$'))?;
    (!var.is_empty()
        && var
            .chars()
            .all(|c| c.is_alphanumeric() || "._-".contains(c)))
    .then_some(var)
}

/// Where each byte of a quoted `value` starts in the `raw` text it was decoded from, and one more
/// for its end, so a match in the value spans its source. A regex cannot see through a backslash
/// escape, such as TOML's `"nightly\u002d2024\u002d01\u002d31"`, which `\uXXXX`, `\UXXXXXXXX`,
/// and `\xXX` (toml.io/en/v1.0.0#string, yaml.org/spec/1.2.2/#57-escaped-characters) or one
/// escaped character write. `None` where the value is not the raw text with such escapes.
fn unescaped(raw: &str, value: &str) -> Option<Vec<usize>> {
    if raw == value {
        return Some((0..=value.len()).collect());
    }
    let mut at = Vec::with_capacity(value.len() + 1);
    let mut chars = value.chars();
    let mut i = 0;
    while i < raw.len() {
        let c = chars.next()?;
        let width = match raw[i..].strip_prefix('\\').and_then(|e| e.chars().next()) {
            Some('u') => 6,
            Some('U') => 10,
            Some('x') => 4,
            Some(e) => 1 + e.len_utf8(),
            None if raw[i..].starts_with(c) => c.len_utf8(),
            None => return None,
        };
        at.extend(std::iter::repeat_n(i, c.len_utf8()));
        i += width;
    }
    (chars.next().is_none() && i == raw.len()).then(|| {
        at.push(raw.len());
        at
    })
}

fn from_caps(caps: &Captures, text: &str, offset: usize, file: &FileType) -> Option<Dep> {
    let group = |name: &str| caps.get_group_by_name(name).map(|s| &text[s.range()]);
    // A `channel` names a release line that moves on its own, such as nvm's `lts/*` or Bun's
    // `canary`: it spans the version a lens shows, and asks for the newest the line has.
    let channel = caps.get_group_by_name("channel").map(|s| s.range());
    let version = caps
        .get_group_by_name("version")
        .map(|s| s.range())
        .or_else(|| channel.clone());
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
    // A `ref` names the key that holds the version, as `${ref}` would, such as a Gradle
    // catalog's `version.ref`.
    let req = match group("ref") {
        _ if channel.is_some() => String::new(),
        Some(r) => format!("${{{r}}}"),
        None => text[span.clone()].to_owned(),
    };
    Some(Dep {
        name,
        req,
        span: span.start + offset..span.end + offset,
        registry,
        source: group("source").map(str::to_owned),
        min: caps.get_group_by_name("min").is_some(),
        ..Dep::default()
    })
}

mod classify;
mod keys;
mod records;
mod west;

pub(crate) use classify::{DIRECTORY, INSTANCE, PROJECT, SOURCE, local_path, source_label};
use classify::{is_package, is_registry_req};
use keys::keys;
use records::records;

#[cfg(test)]
mod tests;
