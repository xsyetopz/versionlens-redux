//! Every supported manifest as one data row: the files it covers, how to read it, and which
//! registry answers for its dependencies. Supporting a new file type means adding a row to `FILE_TYPES` in `catalog/rows.rs`.

use crate::extract::Rule;
use crate::flatten::Format;

mod rows;

pub(crate) use rows::FILE_TYPES;

/// A kind of manifest.
#[derive(Debug)]
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

const fn row(
    id: &'static str,
    globs: &'static [&'static str],
    format: Format,
    registry: &'static str,
    rules: &'static [Rule],
) -> FileType {
    FileType {
        id,
        globs,
        format,
        registry,
        rules,
        insert: "",
    }
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

/// Whether `pattern` matches the end of `path`, case-insensitively, segment by segment.
pub(crate) fn path_matches(pattern: &str, path: &str) -> bool {
    let lower = path.replace('\\', "/").to_ascii_lowercase();
    let pattern = pattern.to_ascii_lowercase();
    let segments: Vec<&str> = lower.split('/').collect();
    let parts: Vec<&str> = pattern.split('/').collect();
    segments.len() >= parts.len()
        && parts
            .iter()
            .rev()
            .zip(segments.iter().rev())
            .all(|(p, s)| glob(p, s))
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
