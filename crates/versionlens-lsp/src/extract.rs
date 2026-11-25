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
    /// A rule read only inside each match of a regex, or of its first group where it has one,
    /// because a bare name means a dependency only in its list: `"lwt"` in an opam `depends`
    /// list, not in `tags`. Regexes have no lookbehind, so a lines rule alone cannot tell them
    /// apart.
    Within(&'static str, &'static Self),
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
    const SCHEMES: &[&str] = &["file://", "file:", "link:", "portal:", "path:"];
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

/// The registry id of a manifest's own version, which gets the next releases as updates, as
/// upstream's project version lenses offer, rather than a lookup.
pub(crate) const PROJECT: &str = "project";

/// The registry id of dependencies on a source no registry answers, such as a git repository or a
/// tarball URL. They need no lookup; their lens names the source, as the fork's master did.
pub(crate) const SOURCE: &str = "source";

/// The kind of source a requirement names when no registry answers it, labeled as the fork's
/// master labeled it: hosted URLs, `.git` paths, and SSH forms are git repositories, other URLs
/// are package URLs, and `workspace:` and `catalog:` point inside the repository.
pub(crate) fn source_label(req: &str) -> Option<&'static str> {
    const GIT: &[&str] = &[
        "git+",
        "git@",
        "git://",
        "ssh://",
        "github:",
        "gitlab:",
        "bitbucket:",
    ];
    const HOSTS: &[&str] = &["github.com", "gitlab.com", "bitbucket.org"];
    let req = req.trim();
    let lower = req.to_ascii_lowercase();
    let host = ["https://", "http://"]
        .iter()
        .find_map(|scheme| lower.strip_prefix(scheme))
        .and_then(|rest| rest.split('/').next())
        // The authority may carry a user and a port: `https://user@github.com:443/o/a`.
        .and_then(|authority| authority.rsplit('@').next())
        .and_then(|host| host.split(':').next());
    let bare = lower.split(['#', '?']).next().unwrap_or(&lower);
    if GIT.iter().any(|p| lower.starts_with(p))
        || std::path::Path::new(bare)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("git"))
        || host.is_some_and(|h| HOSTS.contains(&h) && !is_archive(bare))
    {
        Some("git repository")
    } else if host.is_some() {
        Some("package URL")
    } else if lower.starts_with("workspace:") {
        Some("workspace package")
    } else if lower.starts_with("catalog:") {
        Some("catalog entry")
    } else {
        None
    }
}

/// A release download or archive on a git host, which is a package URL rather than a repository.
fn is_archive(url: &str) -> bool {
    [".tar.gz", ".tgz", ".zip", ".tar.bz2", ".tar.xz", ".whl"]
        .iter()
        .any(|ext| url.ends_with(ext))
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
            | ("esp", "idf")
            // `FROM scratch` starts an empty image: docs.docker.com/build/building/base-images/
            | ("docker", "scratch")
    ) || (name.contains('$') && !is_instance_component(name, registry))
        // An ESP component is `namespace/name`; idf_component.yml's row reads each name once as
        // written and once under the default `espressif/`, and this keeps the one that fits.
        || (registry == "esp" && name.matches('/').count() > 1)
        // A Perl module never starts with `-`; dist.ini's `-phase` and `-relationship` configure
        // the Prereqs plugin itself.
        || (registry == "cpan" && name.starts_with('-'))
        || (registry == "composer"
            && (name.starts_with("ext-")
                || name.starts_with("lib-")
                || name.starts_with("composer-"))))
}

/// A GitLab component or included project on the pipeline's own instance, `$CI_SERVER_FQDN/group/project`, whose
/// variable `project::extract` replaces with the workspace remote's host.
pub(crate) fn is_instance_component(name: &str, registry: &str) -> bool {
    ["gitlab-host", crate::git_ref::GIT_REF].contains(&registry) && name.starts_with(INSTANCE)
}

/// The variable a GitLab pipeline names its own instance by, with the `/` that follows it:
/// <https://docs.gitlab.com/ci/variables/predefined_variables/>.
pub(crate) const INSTANCE: &str = "$CI_SERVER_FQDN/";

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

/// The dependencies the file type's rules find, settled and deduplicated.
fn read(file: &FileType, text: &str) -> Vec<Dep> {
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
                    // A value may list several, as library.properties' `depends=A, B (>=1.0)`.
                    for caps in re.captures_iter(&leaf.value) {
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
                deps.extend(read(&inner, text));
            }
            Rule::Affix(prefix, suffix, rule) => {
                deps.extend(affix(file, prefix, suffix, rule, text));
            }
            Rule::Within(block, rule) => {
                let inner = FileType {
                    rules: std::slice::from_ref(rule),
                    ..*file
                };
                for caps in regex(block).captures_iter(text) {
                    // A block that captures a `suffix`, such as the full Scala version Mill's
                    // `:::` appends, holds its rule in the whole match.
                    let suffix = caps.get_group_by_name("suffix").map(|m| &text[m.range()]);
                    let first = caps.get_group(1).filter(|_| suffix.is_none());
                    let Some(range) = first.or_else(|| caps.get_group(0)) else {
                        continue;
                    };
                    let range = range.range();
                    deps.extend(
                        read(&inner, &text[range.clone()])
                            .into_iter()
                            .map(|mut dep| {
                                dep.span = dep.span.start + range.start..dep.span.end + range.start;
                                if let Some(suffix) = suffix
                                    && ![DIRECTORY, SOURCE].contains(&dep.registry)
                                {
                                    dep.name.push_str(suffix);
                                }
                                dep
                            }),
                    );
                }
            }
        }
    }
    for dep in &mut deps {
        settle(dep, file.pinned, &leaves);
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
    deps
}

/// Settles where a dependency resolves: a local or git source, a registry, or the key a
/// `${ref}` names.
fn settle(dep: &mut Dep, pinned: bool, leaves: &[Leaf]) {
    dep.pinned |= pinned;
    // A Helm chart's `repository` may be a local chart, a `helm repo add` alias, or an OCI
    // registry: helm.sh/docs/helm/helm_dependency/ and helm.sh/docs/topics/registries/
    match dep.source.as_deref() {
        Some(source) if local_path(source).is_some() => dep.registry = DIRECTORY,
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
    if let Some(var) = dep.req.strip_prefix("${").and_then(|r| r.strip_suffix('}'))
        && let Some(leaf) = leaves.iter().find(|l| match &l.path[..] {
            [.., key] if key == var => true,
            [.., key, kind] => key == var && ["require", "strictly", "prefer"].contains(&&**kind),
            _ => false,
        })
    {
        dep.req.clone_from(&leaf.value);
        dep.span = leaf.span.clone();
    }
}

fn keys(map: &str, leaves: &[Leaf], registry: &'static str, deps: &mut Vec<Dep>) {
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
        let (registry, source) = match tail.iter().map(String::as_str).collect::<Vec<_>>()[..] {
            [] | ["version"] => (registry, None),
            ["path"] => (DIRECTORY, None),
            ["git"] | ["git", "url"] => (SOURCE, Some("git repository")),
            ["url"] if source_label(&leaf.value).is_some() => (SOURCE, source_label(&leaf.value)),
            _ => continue,
        };
        deps.push(Dep {
            name: name.clone(),
            req: leaf.value.clone(),
            span: leaf.span.clone(),
            registry,
            source: source.map(str::to_owned),
            index: index(leaf, leaves),
            targets: Vec::new(),
            pinned: false,
            precise: false,
            min: false,
        });
    }
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
            index: None,
            targets: Vec::new(),
            pinned: false,
            precise: false,
            min: false,
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
    // A `ref` names the key that holds the version, as `${ref}` would, such as a Gradle
    // catalog's `version.ref`.
    let req = group("ref").map_or_else(|| text[span.clone()].to_owned(), |r| format!("${{{r}}}"));
    Some(Dep {
        name,
        req,
        span: span.start + offset..span.end + offset,
        registry,
        source: group("source").map(str::to_owned),
        index: None,
        targets: Vec::new(),
        pinned: false,
        precise: false,
        min: caps.get_group_by_name("min").is_some(),
    })
}

mod west;

#[cfg(test)]
mod tests;
