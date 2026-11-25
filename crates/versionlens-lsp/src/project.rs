//! Registry configuration a project and its user keep outside the manifest: `.npmrc`,
//! `nuget.config`, Composer `repositories` and `auth.json`, Python indexes, Cargo
//! `.cargo/config.toml`, and Maven `settings.xml`. Each reader turns it into the URLs a
//! dependency's versions are fetched from, in the order the package manager tries them, with the
//! `Authorization` header each needs.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use crate::extract::Dep;
use crate::flatten::{Format, Leaf, flatten};

mod cargo;
mod composer;
mod maven;
mod npm;
mod nuget;
mod python;

/// A registry URL template a dependency's versions are fetched from, with the `Authorization`
/// header value the project's configuration gives it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Target {
    pub(crate) url: String,
    pub(crate) auth: Option<String>,
    /// The URL is a `NuGet` v3 service index, whose `PackageBaseAddress/3.0.0` resource holds the
    /// version lists.
    pub(crate) index: bool,
}

impl Target {
    fn new(url: String, auth: Option<String>) -> Self {
        Self {
            url,
            auth,
            index: false,
        }
    }
}

/// What the readers see of the machine: environment variables, the home directory, and a way to
/// run a command in a directory. Tests replace all three so no reader touches the user's files.
pub(crate) struct Env {
    pub(crate) vars: HashMap<String, String>,
    pub(crate) home: Option<PathBuf>,
    pub(crate) run: fn(&Path, &str, &[&str]) -> Option<String>,
}

impl Env {
    /// The server process's own environment.
    pub(crate) fn process() -> Self {
        let vars: HashMap<String, String> = std::env::vars().collect();
        let home = vars
            .get("HOME")
            .or_else(|| vars.get("USERPROFILE"))
            .map(PathBuf::from);
        Self {
            vars,
            home,
            run: crate::http::run_in,
        }
    }

    fn var(&self, name: &str) -> Option<&str> {
        self.vars
            .get(name)
            .map(String::as_str)
            .filter(|v| !v.is_empty())
    }

    fn home(&self, path: &str) -> Option<PathBuf> {
        self.home.as_ref().map(|h| h.join(path))
    }
}

fn read(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Sets each dependency's [`Dep::targets`] from the configuration around the manifest at `path`,
/// whose `text` some readers also consult, such as a pyproject's `[[tool.uv.index]]`.
pub(crate) fn resolve(path: &Path, text: &str, deps: &mut [Dep], env: &Env) {
    let Some(dir) = path.parent() else {
        return;
    };
    let registries: BTreeSet<&str> = deps.iter().map(|d| d.registry).collect();
    for registry in registries {
        let targets: Box<Targets<'_>> = match registry {
            "npm" => Box::new(npm::config(dir, env)),
            "nuget" => Box::new(nuget::config(dir, env)),
            "composer" => Box::new(composer::config(dir, text, env)),
            "pypi" => Box::new(python::config(path, text, env)),
            "cargo" => Box::new(cargo::config(dir, env)),
            "maven" => Box::new(maven::config(env)),
            _ => continue,
        };
        for dep in deps.iter_mut().filter(|d| d.registry == registry) {
            dep.targets = targets(dep);
        }
    }
}

/// What a reader returns: the targets for one of its registry's dependencies.
type Targets<'a> = dyn Fn(&Dep) -> Vec<Target> + 'a;

/// A parsed configuration file, read by dotted path.
struct Tree(Vec<Leaf>);

impl Tree {
    fn parse(format: Format, text: &str) -> Self {
        Self(flatten(format, text))
    }

    /// The leaves whose path starts with `prefix.`, as `(rest, value, parent)`.
    fn under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (String, &'a str, usize)> {
        self.0.iter().filter_map(move |l| {
            let path = joined(&l.path);
            let rest = path.strip_prefix(prefix)?.strip_prefix('.')?.to_owned();
            Some((rest, l.value.as_str(), l.parent))
        })
    }

    /// The objects under `prefix`, each as its field names and values, in document order. Fields
    /// are last path segments, so a key with dots in it, such as a host, stays whole.
    fn records(&self, prefix: &str) -> Vec<HashMap<String, String>> {
        let prefix: Vec<&str> = prefix.split('.').collect();
        let mut out: Vec<(usize, HashMap<String, String>)> = Vec::new();
        for leaf in &self.0 {
            let (Some(head), Some(field)) = (leaf.path.get(..prefix.len()), leaf.path.last())
            else {
                continue;
            };
            if leaf.path.len() <= prefix.len() || head.iter().zip(&prefix).any(|(a, b)| a != b) {
                continue;
            }
            let (field, value, parent) = (field.clone(), leaf.value.as_str(), leaf.parent);
            match out.iter_mut().find(|(id, _)| *id == parent) {
                Some((_, record)) => {
                    record.entry(field).or_insert_with(|| value.to_owned());
                }
                None => out.push((parent, HashMap::from([(field, value.to_owned())]))),
            }
        }
        out.into_iter().map(|(_, record)| record).collect()
    }
}

fn joined(path: &[String]) -> String {
    path.join(".")
}

/// The URL without its trailing slashes, ready for a path template to follow.
fn base(url: &str) -> &str {
    url.trim().trim_end_matches('/')
}

/// The `Basic` header value for a user and password.
fn basic(user: &str, password: &str) -> String {
    format!("Basic {}", base64(format!("{user}:{password}").as_bytes()))
}

/// Standard padded base64, for the Basic header.
pub(crate) fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            out.push(if i <= chunk.len() {
                char::from(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize])
            } else {
                '='
            });
        }
    }
    out
}

/// Decodes standard base64, ignoring padding and whitespace; `None` on any other character.
fn unbase64(text: &str) -> Option<String> {
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for c in text
        .bytes()
        .filter(|c| !c.is_ascii_whitespace() && *c != b'=')
    {
        let value = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            _ => return None,
        };
        bits = (bits << 6) | u32::from(value);
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push(u8::try_from((bits >> count) & 0xff).ok()?);
        }
    }
    String::from_utf8(out).ok()
}

/// Replaces each `open NAME close` with the variable's value, leaving unknown names as written,
/// as npm does for `${NAME}` and `NuGet` for `%NAME%`.
fn substitute(text: &str, open: &str, close: &str, env: &Env) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(start) = rest.find(open) {
        let after = &rest[start + open.len()..];
        let Some(end) = after.find(close) else {
            break;
        };
        out.push_str(&rest[..start]);
        let name = &after[..end];
        match env.var(name) {
            Some(value) => out.push_str(value),
            None => out.push_str(&rest[start..start + open.len() + end + close.len()]),
        }
        rest = &after[end + close.len()..];
    }
    out.push_str(rest);
    out
}

/// Whether a package name matches a pattern that may end in `*`, as Composer's `only` and
/// `NuGet`'s `packageSourceMapping` write them.
fn glob(pattern: &str, name: &str) -> bool {
    let (pattern, name) = (pattern.to_ascii_lowercase(), name.to_ascii_lowercase());
    match pattern.strip_suffix('*') {
        Some(prefix) => name.starts_with(prefix),
        None => pattern == name,
    }
}

#[cfg(test)]
mod tests;
