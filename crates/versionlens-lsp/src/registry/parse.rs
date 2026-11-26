//! Reads registry responses into version lists, with the versions each registry marks yanked or
//! deprecated kept apart.

use std::collections::HashSet;

use serde_json::Value;

use super::{Parse, Registry, Versions};
use crate::extract::regex;

impl From<Vec<String>> for Versions {
    fn from(list: Vec<String>) -> Self {
        Self {
            list,
            ..Self::default()
        }
    }
}

/// The versions each match of `pattern` captures. With two groups, the first names the package,
/// for pages that also link other packages. A page may name a release more than once, as Racket's
/// links its installers and its docs, so each version is kept once, where it first appears.
fn matched(pattern: &'static str, body: &str, name: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    regex(pattern)
        .captures_iter(body)
        .filter_map(|c| match (c.get_group(1), c.get_group(2)) {
            (Some(owner), Some(version)) => {
                (&body[owner.range()] == name).then(|| body[version.range()].to_owned())
            }
            (Some(version), None) => Some(body[version.range()].to_owned()),
            _ => None,
        })
        .filter(|version| seen.insert(version.clone()))
        .collect()
}

impl Registry {
    /// Reads a response body into versions.
    ///
    /// # Errors
    /// Explains a body that is not the expected shape.
    pub(crate) fn parse(&self, body: &str, name: &str) -> Result<Versions, String> {
        match &self.parse {
            // With two groups, the first names the package, for pages that also link other packages.
            // A page may name a release more than once, as Racket's links its installers and its
            // docs, so each version is kept once, where it first appears.
            Parse::Regex(pattern) => Ok(matched(pattern, body, name).into()),
            Parse::Fn(read) => Ok(read(body, name)),
            Parse::Json { .. } if !self.html.is_empty() && body.trim_start().starts_with('<') => {
                Ok(matched(self.html, body, name).into())
            }
            Parse::Json { at, field, latest } => {
                let root: Value =
                    serde_json::from_str(body).map_err(|e| format!("invalid JSON: {e}"))?;
                let node =
                    pointer(&root, at, name).ok_or_else(|| format!("no `{at}` in the response"))?;
                let latest = pointer(&root, latest, name)
                    .filter(|_| !latest.is_empty())
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                let (yanked, deprecated) = (self.yanked, self.deprecated);
                Ok(Versions {
                    list: elements(node, field, &|v| !marked(v, yanked)),
                    yanked: elements(node, field, &|v| marked(v, yanked)),
                    deprecated: elements(node, field, &|v| marked(v, deprecated)),
                    latest,
                })
            }
        }
    }
}

/// Whether a version's entry carries `mark` as `true` or as a non-empty message, the way npm
/// writes `deprecated` and JSR `yanked`; an empty `mark` marks nothing.
fn marked(entry: &Value, mark: &str) -> bool {
    !mark.is_empty()
        && entry.get(mark).is_some_and(|m| {
            m.as_bool()
                .unwrap_or_else(|| m.as_str().is_some_and(|s| !s.is_empty()))
        })
}

fn pointer<'a>(root: &'a Value, path: &str, name: &str) -> Option<&'a Value> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .try_fold(root, |node, seg| {
            let seg = if seg == "{name}" { name } else { seg };
            match node {
                Value::Array(items) => items.get(seg.parse::<usize>().ok()?),
                _ => node.get(seg),
            }
        })
}

/// The versions under `node` whose entries pass `keep`.
fn elements(node: &Value, field: &str, keep: &dyn Fn(&Value) -> bool) -> Vec<String> {
    let text = |v: &Value| {
        v.as_str()
            .map(str::to_owned)
            .or_else(|| v.as_number().map(ToString::to_string))
    };
    let pick = |v: &Value| {
        if !keep(v) {
            None
        } else if field.is_empty() {
            text(v)
        } else {
            field.split('|').find_map(|f| v.get(f).and_then(text))
        }
    };
    match node {
        Value::Object(map) if field == "@key" => map
            .iter()
            .filter(|(_, v)| keep(v))
            .map(|(k, _)| k.clone())
            .collect(),
        Value::Object(map) => map.values().filter_map(pick).collect(),
        Value::Array(items) => items.iter().filter_map(pick).collect(),
        other if keep(other) => text(other).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// The crates.io index holds one JSON line per release with its `vers`. A crates.io-compatible
/// web API answers `{apiUrl}/{name}/versions` with one object whose `versions[]` carry `num`, the
/// shape upstream 1.3.0's cratesClient.ts read. Yanked releases go to their own list.
pub(super) fn cargo(body: &str, _name: &str) -> Versions {
    let entries = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v.get("versions")?.as_array().cloned())
        .unwrap_or_else(|| {
            body.lines()
                .filter_map(|l| serde_json::from_str(l).ok())
                .collect()
        });
    let mut out = Versions::default();
    for entry in entries {
        let Some(vers) = entry
            .get("vers")
            .or_else(|| entry.get("num"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let list = if marked(&entry, "yanked") {
            &mut out.yanked
        } else {
            &mut out.list
        };
        list.push(vers.to_owned());
    }
    out
}

/// A Helm repository index lists every chart; this reads the `version` of each entry of one.
pub(super) fn helm(body: &str, name: &str) -> Versions {
    let header = format!("{name}:");
    let mut lines = body
        .lines()
        .skip_while(|l| l.trim_start() != header || l.len() - l.trim_start().len() > 2)
        .skip(1);
    let mut item = None;
    let mut out = Vec::new();
    for line in lines.by_ref() {
        let indent = line.len() - line.trim_start().len();
        let content = line.trim_start();
        if content.is_empty() {
            continue;
        }
        if indent <= 2 && !content.starts_with('-') {
            break;
        }
        let (at, content) = match content.strip_prefix("- ") {
            Some(rest) => (indent + 2, rest),
            None => (indent, content),
        };
        let item_indent = *item.get_or_insert(at);
        if at == item_indent
            && let Some(version) = content.strip_prefix("version:")
        {
            out.push(version.trim().trim_matches(['"', '\'']).to_owned());
        }
    }
    out.into()
}

/// The Go release feed names versions `go1.22.1`; requirements and `.go-version` drop the prefix.
pub(super) fn go_releases(body: &str, _name: &str) -> Versions {
    let Ok(Value::Array(items)) = serde_json::from_str::<Value>(body) else {
        return Versions::default();
    };
    items
        .iter()
        .filter_map(|item| item.get("version")?.as_str()?.strip_prefix("go"))
        .map(str::to_owned)
        .collect::<Vec<_>>()
        .into()
}

/// Terraform's provider endpoint lists `versions`; its module endpoint nests them in `modules`.
pub(super) fn terraform(body: &str, _name: &str) -> Versions {
    let Ok(root) = serde_json::from_str::<Value>(body) else {
        return Versions::default();
    };
    let versions = root
        .get("versions")
        .or_else(|| root.pointer("/modules/0/versions"));
    let Some(items) = versions.and_then(Value::as_array) else {
        return Versions::default();
    };
    items
        .iter()
        .filter_map(|item| item.get("version")?.as_str().map(str::to_owned))
        .collect::<Vec<_>>()
        .into()
}

/// The reader a catalog row names with `reader`, for a response neither JSON paths nor a regex
/// can read.
pub(super) fn reader(name: &str) -> Option<fn(&str, &str) -> Versions> {
    Some(match name {
        "cargo" => cargo,
        "helm" => helm,
        "terraform" => terraform,
        "go-releases" => go_releases,
        _ => return None,
    })
}
