//! Composer's repositories and credentials, as <https://getcomposer.org/doc/05-repositories.md>
//! and <https://getcomposer.org/doc/articles/authentication-for-private-packages.md> document
//! them. `composer` repositories in the manifest are asked first, in order, each limited by its
//! `only` and `exclude` patterns, and Packagist last unless `"packagist.org": false` turns it off.
//! Credentials come from `COMPOSER_AUTH`, the `auth.json` beside the manifest, then the one in
//! `COMPOSER_HOME` (`~/.config/composer` or `~/.composer` when unset): `http-basic.{host}` gives
//! a user and password, `bearer.{host}` a token.

use std::path::Path;

use super::{Env, Target, Tree, base, basic, glob, read};
use crate::extract::Dep;
use crate::flatten::Format;

const PACKAGIST: &str = "https://repo.packagist.org/p2/{name}.json";

struct Repository {
    url: String,
    only: Vec<String>,
    exclude: Vec<String>,
}

pub(super) fn config(dir: &Path, text: &str, env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let tree = Tree::parse(Format::Json, text);
    // The flattened tree keeps strings and numbers only, so the `false` is read from the JSON.
    let json: serde_json::Value = serde_json::from_str(text).unwrap_or_default();
    let off = |v: &serde_json::Value| {
        ["packagist.org", "packagist"]
            .iter()
            .any(|k| v.get(k) == Some(&serde_json::Value::Bool(false)))
    };
    let repositories = &json["repositories"];
    let packagist = !(off(repositories) || repositories.as_array().into_iter().flatten().any(off));
    let field = |id: &str, name: &str| -> Vec<String> {
        tree.0
            .iter()
            .filter(|l| {
                l.path.len() >= 3
                    && l.path[0] == "repositories"
                    && l.path[1] == id
                    && l.path[2] == name
            })
            .map(|l| l.value.clone())
            .collect()
    };
    let repositories: Vec<Repository> = tree
        .0
        .iter()
        .filter(|l| l.path.len() == 3 && l.path[0] == "repositories" && l.path[2] == "url")
        .filter(|l| field(&l.path[1], "type") == ["composer"])
        .map(|l| Repository {
            url: l.value.clone(),
            only: field(&l.path[1], "only"),
            exclude: field(&l.path[1], "exclude"),
        })
        .collect();
    let mut auth_texts: Vec<String> = env
        .var("COMPOSER_AUTH")
        .map(str::to_owned)
        .into_iter()
        .collect();
    let home = env
        .var("COMPOSER_HOME")
        .map(Into::into)
        .or_else(|| env.home(".config/composer").filter(|p| p.is_dir()))
        .or_else(|| env.home(".composer"));
    auth_texts.extend(
        [
            Some(dir.join("auth.json")),
            home.map(|h| h.join("auth.json")),
        ]
        .into_iter()
        .flatten()
        .filter_map(|p| read(&p)),
    );
    let auth: Vec<(String, String)> = auth_texts.iter().flat_map(|t| credentials(t)).collect();
    move |dep: &Dep| {
        let mut targets: Vec<Target> = repositories
            .iter()
            .filter(|r| r.only.is_empty() || r.only.iter().any(|p| glob(p, &dep.name)))
            .filter(|r| !r.exclude.iter().any(|p| glob(p, &dep.name)))
            .map(|r| {
                let host = host(&r.url);
                let header = auth
                    .iter()
                    .find(|(h, _)| *h == host)
                    .map(|(_, v)| v.clone());
                Target::new(format!("{}/p2/{{name}}.json", base(&r.url)), header)
            })
            .collect();
        if targets.is_empty() {
            return targets;
        }
        if packagist {
            targets.push(Target::new(PACKAGIST.to_owned(), None));
        }
        targets
    }
}

fn host(url: &str) -> &str {
    let rest = url.split_once("://").map_or(url, |(_, r)| r);
    rest.split(['/', ':']).next().unwrap_or(rest)
}

/// `(host, header)` pairs from an `auth.json` text.
fn credentials(text: &str) -> Vec<(String, String)> {
    let tree = Tree::parse(Format::Json, text);
    let mut out = Vec::new();
    for leaf in &tree.0 {
        match &leaf.path[..] {
            [kind, host] if kind == "bearer" => {
                out.push((host.clone(), format!("Bearer {}", leaf.value)));
            }
            [kind, host, field] if kind == "http-basic" && field == "username" => {
                let password = tree.0.iter().find(|l| {
                    l.path.len() == 3
                        && l.path[0] == "http-basic"
                        && l.path[1] == *host
                        && l.path[2] == "password"
                });
                if let Some(password) = password {
                    out.push((host.clone(), basic(&leaf.value, &password.value)));
                }
            }
            _ => {}
        }
    }
    out
}
