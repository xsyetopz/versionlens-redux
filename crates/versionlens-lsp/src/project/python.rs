//! Python package indexes. A dependency that names its index (Poetry's `source`, Pipenv's
//! `index`, or uv's `tool.uv.sources.<package>.index`) is looked up there; others go to the
//! default index. Sources, cited per reader:
//! - uv, <https://docs.astral.sh/uv/concepts/indexes/>: `[[tool.uv.index]]` with `name`, `url`,
//!   `default`, and `explicit`; non-explicit indexes come before `PyPI` and a `default` one
//!   replaces it; `UV_DEFAULT_INDEX` and `UV_INDEX` add them from the environment, and
//!   `UV_INDEX_<NAME>_USERNAME` and `_PASSWORD` authenticate one.
//! - Poetry, <https://python-poetry.org/docs/repositories/>: `[[tool.poetry.source]]` with `name`,
//!   `url`, and `priority` (`primary` when unset); primary sources come before `PyPI`, explicit
//!   ones only serve dependencies that name them; `POETRY_HTTP_BASIC_<NAME>_USERNAME` and
//!   `_PASSWORD` authenticate one.
//! - Pipenv, <https://pipenv.pypa.io/en/latest/indexes.html>: the first `[[source]]` is the index
//!   for every package that names no other.
//! - pip, <https://pip.pypa.io/en/stable/topics/configuration/>: `--index-url` and
//!   `--extra-index-url` lines in a requirements file, then `PIP_INDEX_URL` and
//!   `PIP_EXTRA_INDEX_URL`, then `index-url` and `extra-index-url` in the `pip.conf` files
//!   (`$VIRTUAL_ENV`, `PIP_CONFIG_FILE`, `~/.config/pip`, `~/Library/Application Support/pip`,
//!   `~/.pip`, and `%APPDATA%\pip\pip.ini`).
//!
//! Credentials written into an index URL (`https://user:pass@host/simple`) become a Basic header.

use std::path::Path;

use super::npm::ini;
use super::{Env, Target, Tree, base, basic, read};
use crate::extract::Dep;
use crate::flatten::Format;

const PYPI: &str = "https://pypi.org/simple";

struct Index {
    name: String,
    url: String,
    /// Serves only the dependencies that name it.
    explicit: bool,
    /// Replaces `PyPI` rather than coming before it.
    default: bool,
}

pub(super) fn config(path: &Path, text: &str, env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let (mut indexes, pinned) = declared(path, text);
    for (var, default) in [
        ("UV_DEFAULT_INDEX", true),
        ("UV_INDEX_URL", true),
        ("UV_INDEX", false),
        ("UV_EXTRA_INDEX_URL", false),
    ] {
        for url in env.var(var).into_iter().flat_map(str::split_whitespace) {
            let (name, url) = url
                .split_once('=')
                .filter(|(n, _)| !n.contains(':'))
                .unwrap_or(("", url));
            indexes.push(Index {
                name: name.to_owned(),
                url: url.to_owned(),
                explicit: false,
                default,
            });
        }
    }
    if !indexes.iter().any(|i| i.default) {
        let (index, extra) = pip(env);
        if let Some(url) = index {
            indexes.push(Index {
                name: String::new(),
                url,
                explicit: false,
                default: true,
            });
        }
        indexes.extend(extra.into_iter().map(|url| Index {
            name: String::new(),
            url,
            explicit: false,
            default: false,
        }));
    }
    let target = |index: &Index| {
        let (url, mut auth) = split_userinfo(&index.url);
        if auth.is_none() && !index.name.is_empty() {
            let key = index
                .name
                .to_ascii_uppercase()
                .replace(|c: char| !c.is_ascii_alphanumeric(), "_");
            auth = ["UV_INDEX", "POETRY_HTTP_BASIC"].iter().find_map(|tool| {
                Some(basic(
                    env.var(&format!("{tool}_{key}_USERNAME"))?,
                    env.var(&format!("{tool}_{key}_PASSWORD"))
                        .unwrap_or_default(),
                ))
            });
        }
        Target::new(format!("{}/{{name}}/", base(&url)), auth)
    };
    let named: Vec<(String, Target)> = indexes
        .iter()
        .filter(|i| !i.name.is_empty())
        .map(|i| (i.name.clone(), target(i)))
        .collect();
    let mut general: Vec<Target> = indexes
        .iter()
        .filter(|i| !i.explicit)
        .map(&target)
        .collect();
    if !indexes.iter().any(|i| i.default && !i.explicit) && !general.is_empty() {
        general.push(Target::new(format!("{PYPI}/{{name}}/"), None));
    }
    move |dep: &Dep| {
        let wanted = dep.index.clone().or_else(|| {
            pinned
                .iter()
                .find(|(p, _)| *p == normalize(&dep.name))
                .map(|(_, i)| i.clone())
        });
        match wanted.and_then(|w| named.iter().find(|(n, _)| *n == w)) {
            Some((_, target)) => vec![target.clone()],
            None if general
                .iter()
                .all(|t| t.url == format!("{PYPI}/{{name}}/") && t.auth.is_none()) =>
            {
                vec![]
            }
            None => general.clone(),
        }
    }
}

/// The indexes the manifest declares, in order, and the packages `tool.uv.sources` pins to one.
fn declared(path: &Path, text: &str) -> (Vec<Index>, Vec<(String, String)>) {
    let file = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    let mut indexes = Vec::new();
    let mut pinned: Vec<(String, String)> = Vec::new();
    if file == "Pipfile" {
        let tree = Tree::parse(Format::Toml, text);
        for (i, source) in tree.records("source").into_iter().enumerate() {
            if let (Some(name), Some(url)) = (source.get("name"), source.get("url")) {
                indexes.push(Index {
                    name: name.clone(),
                    url: url.clone(),
                    explicit: i > 0,
                    default: i == 0,
                });
            }
        }
    } else if path.extension().is_some_and(|e| e == "toml") {
        let tree = Tree::parse(Format::Toml, text);
        // The flattened tree keeps strings only, so `explicit` and `default` are read here.
        let doc: Option<toml_edit::DocumentMut> = text.parse().ok();
        let uv = doc
            .as_ref()
            .and_then(|d| d.get("tool")?.get("uv")?.get("index")?.as_array_of_tables());
        for index in uv.into_iter().flatten() {
            let text = |key: &str| index.get(key).and_then(|v| v.as_str()).map(str::to_owned);
            let flag = |key: &str| index.get(key).and_then(toml_edit::Item::as_bool) == Some(true);
            if let (Some(name), Some(url)) = (text("name"), text("url")) {
                indexes.push(Index {
                    name,
                    url,
                    explicit: flag("explicit"),
                    default: flag("default"),
                });
            }
        }
        for source in tree.records("tool.poetry.source") {
            if let (Some(name), Some(url)) = (source.get("name"), source.get("url")) {
                let priority = source.get("priority").map_or("primary", String::as_str);
                indexes.push(Index {
                    name: name.clone(),
                    url: url.clone(),
                    explicit: priority != "primary",
                    default: false,
                });
            }
        }
        for leaf in &tree.0 {
            if let [head @ .., package, last] = &leaf.path[..]
                && head == ["tool", "uv", "sources"]
                && last == "index"
            {
                pinned.push((normalize(package), leaf.value.clone()));
            }
        }
    } else {
        for line in text.lines().map(str::trim) {
            let option = |names: &[&str]| {
                names.iter().find_map(|n| {
                    let rest = line.strip_prefix(n)?;
                    let rest = rest.strip_prefix('=').unwrap_or(rest);
                    Some(rest.trim().to_owned()).filter(|r| !r.is_empty())
                })
            };
            if let Some(url) = option(&["--index-url", "-i"]) {
                indexes.push(Index {
                    name: String::new(),
                    url,
                    explicit: false,
                    default: true,
                });
            } else if let Some(url) = option(&["--extra-index-url"]) {
                indexes.push(Index {
                    name: String::new(),
                    url,
                    explicit: false,
                    default: false,
                });
            }
        }
    }
    (indexes, pinned)
}

/// PEP 503's normalized project name.
fn normalize(name: &str) -> String {
    name.to_ascii_lowercase().replace(['_', '.'], "-")
}

/// The URL without its `user:password@`, and the Basic header those make.
fn split_userinfo(url: &str) -> (String, Option<String>) {
    let Some((scheme, rest)) = url.split_once("://") else {
        return (url.to_owned(), None);
    };
    let authority = rest.split('/').next().unwrap_or(rest);
    let Some((userinfo, host)) = authority.rsplit_once('@') else {
        return (url.to_owned(), None);
    };
    let (user, password) = userinfo.split_once(':').unwrap_or((userinfo, ""));
    let path = &rest[authority.len()..];
    (
        format!("{scheme}://{host}{path}"),
        Some(basic(user, password)),
    )
}

/// pip's `index-url` and `extra-index-url`: the environment's, else the last configuration file's.
fn pip(env: &Env) -> (Option<String>, Vec<String>) {
    let extra = |text: &str| {
        text.split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    if let Some(url) = env.var("PIP_INDEX_URL") {
        return (
            Some(url.to_owned()),
            env.var("PIP_EXTRA_INDEX_URL").map_or_default(extra),
        );
    }
    let files = [
        env.var("APPDATA").map(|d| Path::new(d).join("pip/pip.ini")),
        env.home(".pip/pip.conf"),
        env.home("Library/Application Support/pip/pip.conf"),
        env.var("XDG_CONFIG_HOME")
            .map(|d| Path::new(d).join("pip/pip.conf"))
            .or_else(|| env.home(".config/pip/pip.conf")),
        env.var("VIRTUAL_ENV")
            .map(|d| Path::new(d).join("pip.conf")),
        env.var("PIP_CONFIG_FILE").map(Into::into),
    ];
    let (mut index, mut more) = (None, Vec::new());
    for text in files.into_iter().flatten().filter_map(|p| read(&p)) {
        for (key, value) in ini(&text) {
            match key.as_str() {
                "index-url" => index = Some(value),
                "extra-index-url" => more = extra(&value),
                _ => {}
            }
        }
    }
    (index, more)
}
