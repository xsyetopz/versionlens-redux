//! Every supported manifest as one row of `catalog/files/`: the files it covers, how to read it,
//! and which registry answers for its dependencies. Supporting a new file type means adding a
//! row to the file for its group of ecosystems.

use std::sync::LazyLock;

use crate::data::{self, Row};
use crate::extract::Rule;
use crate::flatten::Format;

/// The file groups in matching order: a glob two rows share belongs to the earlier row.
static GROUPS: &[(&str, &str)] = &[
    (
        "files/native.toml",
        include_str!("../catalog/files/native.toml"),
    ),
    ("files/js.toml", include_str!("../catalog/files/js.toml")),
    (
        "files/python.toml",
        include_str!("../catalog/files/python.toml"),
    ),
    ("files/jvm.toml", include_str!("../catalog/files/jvm.toml")),
    (
        "files/functional.toml",
        include_str!("../catalog/files/functional.toml"),
    ),
    (
        "files/infra.toml",
        include_str!("../catalog/files/infra.toml"),
    ),
    (
        "files/tooling.toml",
        include_str!("../catalog/files/tooling.toml"),
    ),
];

/// Every file type, in matching order.
pub(crate) static FILE_TYPES: LazyLock<Vec<FileType>> =
    LazyLock::new(|| load().expect("the embedded file-type catalog loads"));

/// A kind of manifest.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FileType {
    pub(crate) id: &'static str,
    /// Matched against the end of the path, case-insensitively; `*` matches within one segment.
    pub(crate) globs: &'static [&'static str],
    pub(crate) format: Format,
    pub(crate) registry: &'static str,
    pub(crate) rules: &'static [Rule],
    /// Written before a version added to a bare name, such as `==` in requirements.txt.
    pub(crate) insert: &'static str,
}

fn load() -> Result<Vec<FileType>, String> {
    let mut all = Vec::new();
    for (name, text) in GROUPS {
        for row in data::rows(name, text, "file")? {
            all.push(file(&row)?);
        }
    }
    Ok(all)
}

/// One `[[file]]` row.
fn file(row: &Row) -> Result<FileType, String> {
    row.only(&["id", "globs", "format", "registry", "insert", "rules"])?;
    let format = match row.need("format")? {
        "json" => Format::Json,
        "toml" => Format::Toml,
        "yaml" => Format::Yaml,
        "xml" => Format::Xml,
        "text" => Format::Text,
        other => {
            return Err(row.error(&format!(
                "format {other} is not json, toml, yaml, xml, or text"
            )));
        }
    };
    let mut rules = Vec::new();
    for entry in row.tables("rules")? {
        // `{ use = 'NAME' }` stands for the shared rule list NAME.
        match entry.opt("use")? {
            Some(name) => {
                entry.only(&["use"])?;
                for shared in data::shared_rules(name)? {
                    rules.push(rule(&shared)?);
                }
            }
            None => rules.push(rule(&entry)?),
        }
    }
    Ok(FileType {
        id: row.need("id")?,
        globs: row.list("globs")?.leak(),
        format,
        registry: row.need("registry")?,
        rules: rules.leak(),
        insert: row.text("insert")?,
    })
}

/// One rule: `keys`, `spec` with its `pattern`, `record` with its `name`, `version`, and
/// optional `source` fields, or `lines`. A `via` names the registry its dependencies go to.
fn rule(entry: &Row) -> Result<Rule, String> {
    entry.only(&[
        "keys", "spec", "pattern", "record", "name", "version", "source", "lines", "via",
    ])?;
    let rule = if let Some(path) = entry.opt("keys")? {
        Rule::Keys(path)
    } else if let Some(path) = entry.opt("spec")? {
        Rule::Spec(path, entry.need("pattern")?)
    } else if let Some(path) = entry.opt("record")? {
        Rule::Record(
            path,
            entry.need("name")?,
            entry.need("version")?,
            entry.text("source")?,
        )
    } else if let Some(pattern) = entry.opt("lines")? {
        Rule::Lines(pattern)
    } else {
        return Err(entry.error("a rule needs keys, spec, record, or lines"));
    };
    Ok(match entry.opt("via")? {
        Some(registry) => Rule::Via(registry, Box::leak(Box::new(rule))),
        None => rule,
    })
}

fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => {
            text.starts_with(head)
                && (head.len()..=text.len())
                    .any(|i| text.is_char_boundary(i) && glob(tail, &text[i..]))
        }
    }
}

/// Matches path segments against pattern segments from the end; a `**` segment spans any
/// number of directories, so `.github/workflows/**/*.yml` reaches nested workflow folders.
fn tail_matches(parts: &[&str], segments: &[&str]) -> bool {
    match parts.split_last() {
        None => true,
        Some((&"**", rest)) => (0..=segments.len()).any(|n| tail_matches(rest, &segments[..n])),
        Some((part, rest)) => match segments.split_last() {
            Some((segment, before)) => glob(part, segment) && tail_matches(rest, before),
            None => false,
        },
    }
}

/// Whether `pattern` matches the end of `path`, case-insensitively, segment by segment.
pub(crate) fn path_matches(pattern: &str, path: &str) -> bool {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    let segments: Vec<&str> = lower.split('/').collect();
    let parts: Vec<&str> = pattern.split('/').collect();
    tail_matches(&parts, &segments)
}

impl FileType {
    pub(crate) fn matches(&self, path: &str) -> bool {
        self.globs.iter().any(|pattern| path_matches(pattern, path))
    }
}

/// The file type with this id.
pub(crate) fn by_id(id: &str) -> Option<&'static FileType> {
    FILE_TYPES.iter().find(|f| f.id == id)
}

/// The file type for a path; the first matching row wins.
pub(crate) fn file_type(path: &str) -> Option<&'static FileType> {
    FILE_TYPES.iter().find(|f| f.matches(path))
}

/// Every glob, for editors that pick which files to send to the server.
pub(crate) fn globs() -> Vec<&'static str> {
    FILE_TYPES
        .iter()
        .flat_map(|f| f.globs.iter().copied())
        .collect()
}

/// [`globs`] as the `--file-types` JSON.
pub(crate) fn globs_json() -> String {
    serde_json::json!({ "globs": globs() }).to_string()
}

#[cfg(test)]
mod tests;
