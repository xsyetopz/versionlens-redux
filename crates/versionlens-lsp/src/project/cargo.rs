//! Cargo's registries, as <https://doc.rust-lang.org/cargo/reference/config.html> and
//! <https://doc.rust-lang.org/cargo/reference/registries.html> document them: `.cargo/config.toml`
//! (or `.cargo/config`) in the manifest's folder and each parent, nearer winning, then the one in
//! `CARGO_HOME` (`~/.cargo`). A dependency's `registry = "name"` picks `registries.<name>.index`,
//! and `source.<name>.replace-with` redirects crates.io or a named registry to another
//! registry or `source.<name>.registry` URL. Tokens come from `CARGO_REGISTRIES_<NAME>_TOKEN`
//! (`CARGO_REGISTRY_TOKEN` for crates.io), the configuration, then `credentials.toml`, and are
//! sent as the bare `Authorization` value, as Cargo sends them. Only sparse (`sparse+https://`)
//! indexes answer over HTTP; a git index is left to Cargo.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::{Env, Target, Tree, base, read};
use crate::extract::Dep;
use crate::flatten::Format;

const CRATES_IO: &str = "crates-io";

pub(super) fn config(dir: &Path, env: &Env) -> impl Fn(&Dep) -> Vec<Target> {
    let home = env
        .var("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| env.home(".cargo"));
    let mut files: Vec<PathBuf> = dir
        .ancestors()
        .filter_map(|d| {
            ["config", "config.toml"]
                .iter()
                .map(|f| d.join(".cargo").join(f))
                .find(|p| p.is_file())
        })
        .collect();
    files.extend(
        home.as_ref()
            .map(|h| h.join("config.toml"))
            .filter(|p| p.is_file())
            .or_else(|| home.as_ref().map(|h| h.join("config"))),
    );
    let mut values: HashMap<String, String> = HashMap::new();
    let mut merge = |text: &str| {
        for leaf in Tree::parse(Format::Toml, text).0 {
            values.entry(leaf.path.join(".")).or_insert(leaf.value);
        }
    };
    for text in files.iter().filter_map(|p| read(p)) {
        merge(&text);
    }
    let mut tokens: HashMap<String, String> = HashMap::new();
    if let Some(text) = home
        .as_ref()
        .and_then(|h| read(&h.join("credentials.toml")).or_else(|| read(&h.join("credentials"))))
    {
        for leaf in Tree::parse(Format::Toml, &text).0 {
            match &leaf.path[..] {
                [registries, name, token] if registries == "registries" && token == "token" => {
                    tokens.insert(name.clone(), leaf.value);
                }
                [registry, token] if registry == "registry" && token == "token" => {
                    tokens.insert(CRATES_IO.to_owned(), leaf.value);
                }
                _ => {}
            }
        }
    }
    let vars: HashMap<String, String> = env
        .vars
        .iter()
        .filter(|(k, _)| k.starts_with("CARGO_REGISTRIES_"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    move |dep: &Dep| {
        let var = |name: &str, field: &str| {
            let key = name.to_ascii_uppercase().replace('-', "_");
            vars.get(&format!("CARGO_REGISTRIES_{key}_{field}"))
                .cloned()
        };
        let mut name = dep.index.clone().unwrap_or_else(|| CRATES_IO.to_owned());
        for _ in 0..8 {
            match values.get(&format!("source.{name}.replace-with")) {
                Some(next) => name.clone_from(next),
                None => break,
            }
        }
        if name == CRATES_IO {
            return vec![];
        }
        let index = var(&name, "INDEX")
            .or_else(|| values.get(&format!("registries.{name}.index")).cloned())
            .or_else(|| values.get(&format!("source.{name}.registry")).cloned());
        let Some(url) = index.and_then(|i| i.strip_prefix("sparse+").map(str::to_owned)) else {
            return vec![];
        };
        let token = var(&name, "TOKEN")
            .or_else(|| values.get(&format!("registries.{name}.token")).cloned())
            .or_else(|| tokens.get(&name).cloned());
        vec![Target::new(format!("{}/{{cargo}}", base(&url)), token)]
    }
}
