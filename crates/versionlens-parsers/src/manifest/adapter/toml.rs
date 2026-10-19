//! The TOML adapter: a [`PathNode`] tree over a span-preserving document,
//! and the walks that visit every value with the keys that lead to it.

use std::ops::Range;

use toml_edit::Item::{
    ArrayOfTables as TomlArrayOfTables, None as TomlNone, Table as TomlTable,
    Value as TomlItemValue,
};
use toml_edit::{Document, Item, Key, Table, TableLike, Value as TomlValue};
use versionlens_model::Dependency;

use super::Scalar;
use crate::manifest::path_query::{PathNode, selects};

#[derive(Clone, Copy)]
enum Kind<'a> {
    Item(&'a Item),
    Table(&'a Table),
    Value(&'a TomlValue),
}

/// A TOML item as a node of a path query. Tables, inline tables and dotted
/// keys are walked by key, and arrays and arrays of tables by index.
#[derive(Clone, Copy)]
pub(crate) struct TomlNode<'a> {
    text: &'a str,
    kind: Kind<'a>,
    key: Option<&'a Key>,
}

impl<'a> TomlNode<'a> {
    pub(crate) fn root(text: &'a str, document: &'a Document<String>) -> Self {
        Self {
            text,
            kind: Kind::Table(document.as_table()),
            key: None,
        }
    }

    fn table_like(&self) -> Option<&'a dyn TableLike> {
        match self.kind {
            Kind::Item(item) => item.as_table_like(),
            Kind::Table(table) => Some(table),
            Kind::Value(value) => value
                .as_inline_table()
                .map(|table| -> &dyn TableLike { table }),
        }
    }

    fn value(&self) -> Option<&'a TomlValue> {
        match self.kind {
            Kind::Item(item) => item.as_value(),
            Kind::Table(_) => None,
            Kind::Value(value) => Some(value),
        }
    }

    fn elements(&self) -> Vec<Self> {
        let node = |kind| Self {
            text: self.text,
            kind,
            key: None,
        };
        match self.kind {
            Kind::Item(TomlArrayOfTables(tables)) => tables
                .iter()
                .map(|table| node(Kind::Table(table)))
                .collect(),
            _ => self
                .value()
                .and_then(TomlValue::as_array)
                .map(|array| array.iter().map(|value| node(Kind::Value(value))).collect())
                .unwrap_or_default(),
        }
    }

    /// The string the node holds, with the span of its content.
    pub(crate) fn scalar(&self) -> Option<Scalar> {
        let value = self.value()?;
        let span = value.span()?;
        let raw = self.text.get(span.clone())?;
        let quote = ["\"\"\"", "'''", "\"", "'"]
            .into_iter()
            .find(|quote| raw.starts_with(quote) && raw.ends_with(quote))?;
        Some(Scalar {
            value: value.as_str()?.to_owned(),
            span: span.start + quote.len()..span.end - quote.len(),
        })
    }

    /// The span of the key that holds the node, absent for the root and for
    /// array elements.
    pub(crate) fn key_span(&self) -> Option<Range<usize>> {
        self.key?.span()
    }

    /// The span of the value, including quotes and braces.
    pub(crate) fn value_span(&self) -> Option<Range<usize>> {
        self.value()?.span()
    }

    /// The span of a `[table]` or `[[array of tables]]` entry, header included.
    pub(crate) fn table_span(&self) -> Option<Range<usize>> {
        match self.kind {
            Kind::Table(table) => table.span(),
            _ => None,
        }
    }

    /// Whether the node is a value, such as a string or an inline table,
    /// rather than a `[table]` or `[[array of tables]]`.
    pub(crate) fn is_value(&self) -> bool {
        self.value().is_some()
    }
}

impl PathNode for TomlNode<'_> {
    fn children_named(&self, key: &str) -> Vec<Self> {
        if let Some(table) = self.table_like() {
            return table
                .get_key_value(key)
                .map(|(key, item)| Self {
                    text: self.text,
                    kind: Kind::Item(item),
                    key: Some(key),
                })
                .into_iter()
                .collect();
        }
        key.parse::<usize>()
            .ok()
            .and_then(|index| self.elements().get(index).copied())
            .into_iter()
            .collect()
    }

    fn children(&self) -> Vec<(String, Self)> {
        if let Some(table) = self.table_like() {
            return table
                .iter()
                .filter_map(|(name, item)| {
                    let (key, _) = table.get_key_value(name)?;
                    Some((
                        name.to_owned(),
                        Self {
                            text: self.text,
                            kind: Kind::Item(item),
                            key: Some(key),
                        },
                    ))
                })
                .collect();
        }
        self.elements()
            .into_iter()
            .enumerate()
            .map(|(index, element)| (index.to_string(), element))
            .collect()
    }
}

pub(crate) fn collect_toml_dependencies(
    text: &str,
    mut visit: impl FnMut(&str, &[&Key], &TomlValue, &mut Vec<Dependency>),
) -> Vec<Dependency> {
    let Ok(document) = crate::parse_toml_document(text) else {
        return vec![];
    };

    let mut dependencies = vec![];
    let mut keys = vec![];
    walk_toml_values(document.as_table(), &mut keys, &mut |keys, value| {
        visit(text, keys, value, &mut dependencies);
    });
    dependencies
}

pub(crate) fn walk_toml_values<'a>(
    table: &'a Table,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    for (key_name, item) in table.iter() {
        let Some(key) = table.key(key_name) else {
            continue;
        };

        keys.push(key);
        visit_toml_item(item, keys, visit);
        keys.pop();
    }
}

fn visit_toml_item<'a>(
    item: &'a Item,
    keys: &mut Vec<&'a Key>,
    visit: &mut impl FnMut(&[&'a Key], &'a TomlValue),
) {
    match item {
        TomlItemValue(value) => visit(keys, value),
        TomlTable(child) => walk_toml_values(child, keys, visit),
        TomlArrayOfTables(_) | TomlNone => {}
    }
}

/// The first of `dependency_paths` that selects the table holding the leaf at
/// `keys`. The leaf's own key is not part of its table path.
pub(crate) fn selected_dependency_path<'a>(
    keys: &[&Key],
    dependency_paths: &'a [&str],
) -> Option<&'a str> {
    let (_, table_keys) = keys.split_last()?;
    let table_keys = table_keys.iter().map(|key| key.get()).collect::<Vec<_>>();

    dependency_paths
        .iter()
        .copied()
        .find(|path| selects(&path.split('.').collect::<Vec<_>>(), &table_keys))
}
