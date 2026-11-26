//! Reads the catalog the binary embeds from `catalog/`: TOML rows for file types and registries,
//! and the named patterns and rule lists in `catalog/shared.toml` that rows reuse. A pattern
//! writes `${NAME}` for a shared pattern. The rows load once and live as long as the process, so
//! their text is kept as `&'static str`.

use std::collections::HashMap;
use std::sync::LazyLock;

use toml_edit::{Document, Item, TableLike, Value};

static SHARED: &str = include_str!("../catalog/shared.toml");

/// The shared patterns by name, from `[patterns]` in `catalog/shared.toml`.
static PATTERNS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    let root = parse(SHARED).expect("catalog/shared.toml parses");
    root.get("patterns")
        .and_then(Item::as_table_like)
        .expect("catalog/shared.toml has [patterns]")
        .iter()
        .filter_map(|(k, v)| Some((k.to_owned(), v.as_str()?.to_owned())))
        .collect()
});

/// Keeps a catalog text for the life of the process.
pub(crate) fn keep(text: &str) -> &'static str {
    Box::leak(Box::from(text))
}

fn parse(text: &str) -> Result<toml_edit::Table, String> {
    Document::parse(text)
        .map(|doc| doc.as_table().clone())
        .map_err(|e| e.to_string())
}

/// The tables of `[[key]]` in one catalog file, each read through a [`Row`] that names the file
/// and row in its errors.
pub(crate) fn rows(file: &'static str, text: &str, key: &str) -> Result<Vec<Row>, String> {
    let root = parse(text).map_err(|e| format!("{file}: {e}"))?;
    let tables = root
        .get(key)
        .and_then(Item::as_array_of_tables)
        .ok_or_else(|| format!("{file}: no [[{key}]] rows"))?;
    Ok(tables
        .iter()
        .map(|t| {
            let id = t
                .get("id")
                .and_then(Item::as_str)
                .unwrap_or("(a row without an id)");
            Row::new(file, id, Box::new(t.clone()))
        })
        .collect())
}

/// The shared rule list `name`, from `[rules]` in `catalog/shared.toml`, as rows to read.
pub(crate) fn shared_rules(name: &str) -> Result<Vec<Row>, String> {
    let root = parse(SHARED)?;
    let list = root
        .get("rules")
        .and_then(|r| r.get(name))
        .and_then(Item::as_array)
        .ok_or_else(|| format!("shared.toml: no rule list {name}"))?;
    inline("shared.toml", name, list.iter())
}

/// The inline tables in `values` as rows that take the label of the row holding them.
fn inline<'a>(
    file: &'static str,
    id: &str,
    values: impl Iterator<Item = &'a Value>,
) -> Result<Vec<Row>, String> {
    values
        .map(|v| {
            v.as_inline_table()
                .map(|t| Row::new(file, id, Box::new(t.clone())))
                .ok_or_else(|| format!("{file}: {id}: an entry is not an inline table"))
        })
        .collect()
}

/// One catalog row: a table whose keys a reader takes one at a time, with errors that name the
/// file and the row's `id`, or the `id` of the row an inline table sits in.
pub(crate) struct Row {
    file: &'static str,
    id: String,
    table: Box<dyn TableLike>,
}

impl Row {
    fn new(file: &'static str, id: &str, table: Box<dyn TableLike>) -> Self {
        Self {
            file,
            id: id.to_owned(),
            table,
        }
    }

    /// `message` prefixed with the file and row it is about.
    pub(crate) fn error(&self, message: &str) -> String {
        format!("{}: {}: {message}", self.file, self.id)
    }

    /// Fails on a key outside `known`, so a misspelled key cannot pass for an absent one.
    pub(crate) fn only(&self, known: &[&str]) -> Result<(), String> {
        match self.table.iter().find(|(k, _)| !known.contains(k)) {
            Some((key, _)) => Err(self.error(&format!("unknown key {key}"))),
            None => Ok(()),
        }
    }

    /// The string at `key` with its `${NAME}` patterns filled in, or `None` where it is absent.
    pub(crate) fn opt(&self, key: &str) -> Result<Option<&'static str>, String> {
        let Some(item) = self.table.get(key) else {
            return Ok(None);
        };
        let text = item
            .as_str()
            .ok_or_else(|| self.error(&format!("{key} is not a string")))?;
        self.fill(text).map(|t| Some(keep(&t)))
    }

    /// The string at `key`, or an empty one where it is absent.
    pub(crate) fn text(&self, key: &str) -> Result<&'static str, String> {
        Ok(self.opt(key)?.unwrap_or(""))
    }

    /// The string at `key`, which the row must have.
    pub(crate) fn need(&self, key: &str) -> Result<&'static str, String> {
        self.opt(key)?
            .ok_or_else(|| self.error(&format!("needs {key}")))
    }

    pub(crate) fn flag(&self, key: &str) -> Result<bool, String> {
        self.table.get(key).map_or(Ok(false), |item| {
            item.as_bool()
                .ok_or_else(|| self.error(&format!("{key} is not true or false")))
        })
    }

    /// The strings of the array at `key`.
    pub(crate) fn list(&self, key: &str) -> Result<Vec<&'static str>, String> {
        let Some(item) = self.table.get(key) else {
            return Ok(Vec::new());
        };
        let array = item
            .as_array()
            .ok_or_else(|| self.error(&format!("{key} is not an array")))?;
        array
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(|| self.error(&format!("{key} holds a non-string")))
                    .and_then(|t| self.fill(t).map(|t| keep(&t)))
            })
            .collect()
    }

    /// The inline tables of the array at `key`, as rows.
    pub(crate) fn tables(&self, key: &str) -> Result<Vec<Self>, String> {
        let array = self
            .table
            .get(key)
            .and_then(Item::as_array)
            .ok_or_else(|| self.error(&format!("{key} is not an array")))?;
        inline(self.file, &self.id, array.iter())
    }

    /// The string pairs of the inline table at `key`, such as a registry's pages by host.
    pub(crate) fn pairs(&self, key: &str) -> Result<Vec<(&'static str, &'static str)>, String> {
        let Some(item) = self.table.get(key) else {
            return Ok(Vec::new());
        };
        let table = item
            .as_table_like()
            .ok_or_else(|| self.error(&format!("{key} is not a table")))?;
        table
            .iter()
            .map(|(k, v)| {
                let value = v
                    .as_str()
                    .ok_or_else(|| self.error(&format!("{key}.{k} is not a string")))?;
                Ok((keep(k), keep(value)))
            })
            .collect()
    }

    fn fill(&self, text: &str) -> Result<String, String> {
        let mut out = text.to_owned();
        while let Some(start) = out.find("${") {
            let end = out[start..]
                .find('}')
                .map(|e| start + e)
                .ok_or_else(|| self.error("an unclosed ${"))?;
            let name = &out[start + 2..end];
            let pattern = PATTERNS
                .get(name)
                .ok_or_else(|| self.error(&format!("no shared pattern {name}")))?;
            out.replace_range(start..=end, pattern);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests;
