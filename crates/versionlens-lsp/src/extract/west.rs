//! West manifests name most projects through a remote: the project's `remote` (or the manifest's
//! `defaults.remote`) names an entry of `manifest.remotes`, whose `url-base` joined with the
//! project's `repo-path` or `name` is the repository: zephyrproject-rtos/zephyr,
//! doc/develop/west/manifest.rst. One remote serves many projects, and a regex match cannot refer
//! to another match, so this joins the YAML leaves instead of a catalog rule.

use std::collections::HashMap;

use super::{Dep, SOURCE};
use crate::flatten::Leaf;

/// The projects a remote names, and those that inherit `defaults.revision`, each at its revision:
/// a GitHub release where the repository is on GitHub and the revision looks like a version, and
/// otherwise the git reference it pins.
pub(super) fn projects(leaves: &[Leaf]) -> Vec<Dep> {
    let field = |list: &str, index: &str, key: &str| {
        leaves.iter().find(|l| {
            l.path.len() == 4
                && l.path[0] == "manifest"
                && l.path[1] == list
                && l.path[2] == index
                && l.path[3] == key
        })
    };
    let default_remote = leaves
        .iter()
        .find(|l| l.path == ["manifest", "defaults", "remote"])
        .map(|l| l.value.as_str());
    let mut bases = HashMap::new();
    for leaf in leaves.iter().filter(|l| {
        l.path.len() == 4 && l.path[..2] == ["manifest", "remotes"] && l.path[3] == "name"
    }) {
        if let Some(base) = field("remotes", &leaf.path[2], "url-base") {
            bases.insert(leaf.value.as_str(), base.value.trim_end_matches('/'));
        }
    }
    // A project without a `revision` checks out `defaults.revision`, so that leaf is its span,
    // shared by every project that inherits it; west's own `master` default is written nowhere.
    let default_revision = leaves
        .iter()
        .find(|l| l.path == ["manifest", "defaults", "revision"]);
    let mut deps = Vec::new();
    for leaf in leaves.iter().filter(|l| {
        l.path.len() == 4 && l.path[..2] == ["manifest", "projects"] && l.path[3] == "name"
    }) {
        let at = |key| field("projects", &leaf.path[2], key);
        // A project with its own `url` and `revision` is read by the row's rules.
        let (url, own) = (at("url"), at("revision"));
        let Some(revision) = own
            .or(default_revision)
            .filter(|_| url.is_none() || own.is_none())
        else {
            continue;
        };
        let release = super::regex(r"\Av?\d+\.\d").is_match(revision.value.as_str());
        let (name, registry, source) = if let Some(url) = url {
            let url = url.value.trim_end_matches('/');
            let url = url.strip_suffix(".git").unwrap_or(url);
            match url.strip_prefix("https://github.com/") {
                Some(repo) if release && repo.matches('/').count() == 1 => {
                    (repo.to_owned(), "github", None)
                }
                _ => (url.to_owned(), SOURCE, Some("git repository".to_owned())),
            }
        } else {
            let Some(base) = at("remote")
                .map(|r| r.value.as_str())
                .or(default_remote)
                .and_then(|r| bases.get(r))
            else {
                continue;
            };
            let repo = at("repo-path").unwrap_or(leaf).value.trim_matches('/');
            let owner = base
                .strip_prefix("https://github.com/")
                .filter(|o| !o.contains('/'));
            match owner {
                Some(owner) if release => (format!("{owner}/{repo}"), "github", None),
                _ => (
                    format!("{base}/{repo}"),
                    SOURCE,
                    Some("git repository".to_owned()),
                ),
            }
        };
        deps.push(Dep {
            name,
            req: revision.value.clone(),
            span: revision.span.clone(),
            registry,
            source,
            index: None,
            targets: Vec::new(),
            pinned: false,
            precise: false,
            min: false,
        });
    }
    deps
}
