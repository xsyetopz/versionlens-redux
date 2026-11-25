//! Flattens JSON, TOML, YAML, XML, and INI documents into scalar leaves with byte spans, so one set of
//! path rules reads every structured manifest the same way.

use std::ops::Range;

use crate::extract::regex;

/// Which flattener reads a manifest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Json,
    Toml,
    Yaml,
    Xml,
    Ini,
    /// Read only by line rules.
    Text,
}

/// One scalar value: its path from the root, its text, and where that text sits in the source.
/// `parent` identifies the enclosing object, so sibling fields of one record group together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Leaf {
    pub(crate) path: Vec<String>,
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
    pub(crate) parent: usize,
}

#[derive(Default)]
struct Out {
    leaves: Vec<Leaf>,
    path: Vec<String>,
    ids: usize,
}

impl Out {
    fn id(&mut self) -> usize {
        self.ids += 1;
        self.ids
    }

    fn leaf(&mut self, value: &str, span: Range<usize>, parent: usize) {
        self.leaves.push(Leaf {
            path: self.path.clone(),
            value: value.to_owned(),
            span,
            parent,
        });
    }
}

pub(crate) fn flatten(format: Format, text: &str) -> Vec<Leaf> {
    let mut out = Out::default();
    match format {
        Format::Json => json(text, &mut out),
        Format::Toml => toml(text, &mut out),
        Format::Yaml => yaml(text, &mut out),
        Format::Xml => xml(text, &mut out),
        Format::Ini => ini(text, &mut out),
        Format::Text => {}
    }
    out.leaves
}

/// Shrinks a quoted literal's span to the text between the quotes.
fn unquote(text: &str, span: Range<usize>) -> Range<usize> {
    let raw = &text[span.clone()];
    let quoted = raw.len() >= 2 && raw.starts_with(['"', '\'']) && raw.ends_with(&raw[..1]);
    if quoted {
        span.start + 1..span.end - 1
    } else {
        span
    }
}

fn json(text: &str, out: &mut Out) {
    use jsonc_parser::ast::Value;
    fn walk(value: &Value<'_>, text: &str, parent: usize, out: &mut Out) {
        match value {
            Value::Object(object) => {
                let id = out.id();
                for prop in &object.properties {
                    out.path.push(prop.name.as_str().to_owned());
                    walk(&prop.value, text, id, out);
                    out.path.pop();
                }
            }
            Value::Array(array) => {
                for (index, element) in array.elements.iter().enumerate() {
                    out.path.push(index.to_string());
                    walk(element, text, parent, out);
                    out.path.pop();
                }
            }
            Value::StringLit(lit) => out.leaf(
                &lit.value,
                unquote(text, lit.range.start..lit.range.end),
                parent,
            ),
            Value::NumberLit(lit) => out.leaf(lit.value, lit.range.start..lit.range.end, parent),
            // A flag names its key with no version, as package.json's object-form
            // `bundleDependencies` does: npm/package-json lib/normalize.js reads `Object.keys`.
            Value::BooleanLit(lit) => out.leaf("", lit.range.start..lit.range.start, parent),
            Value::NullKeyword(_) => {}
        }
    }
    let options = jsonc_parser::ParseOptions::default();
    if let Ok(parsed) =
        jsonc_parser::parse_to_ast(text, &jsonc_parser::CollectOptions::default(), &options)
        && let Some(value) = parsed.value
    {
        walk(&value, text, 0, out);
    }
}

fn toml(text: &str, out: &mut Out) {
    use toml_edit::{Item, Value};
    fn value(node: &Value, text: &str, parent: usize, out: &mut Out) {
        match node {
            Value::InlineTable(table) => {
                let id = out.id();
                for (key, inner) in table {
                    out.path.push(key.to_owned());
                    value(inner, text, id, out);
                    out.path.pop();
                }
            }
            Value::Array(array) => {
                for (index, inner) in array.iter().enumerate() {
                    out.path.push(index.to_string());
                    value(inner, text, parent, out);
                    out.path.pop();
                }
            }
            Value::String(s) => {
                if let Some(span) = node.span() {
                    out.leaf(s.value(), unquote(text, span), parent);
                }
            }
            _ => {}
        }
    }
    fn item(node: &Item, text: &str, parent: usize, out: &mut Out) {
        match node {
            Item::Table(table) => {
                let id = out.id();
                for (key, inner) in table {
                    out.path.push(key.to_owned());
                    item(inner, text, id, out);
                    out.path.pop();
                }
            }
            Item::ArrayOfTables(tables) => {
                for (index, table) in tables.iter().enumerate() {
                    out.path.push(index.to_string());
                    let id = out.id();
                    for (key, inner) in table {
                        out.path.push(key.to_owned());
                        item(inner, text, id, out);
                        out.path.pop();
                    }
                    out.path.pop();
                }
            }
            Item::Value(inner) => value(inner, text, parent, out),
            Item::None => {}
        }
    }
    if let Ok(doc) = toml_edit::Document::parse(text) {
        item(doc.as_item(), text, 0, out);
    }
}

/// `key = value` lines under `[section]` headers, as `Config::MVP` reads them for the
/// `dist.ini` of `Dist::Zilla`: a header's ` / ` separates the plugin from its name, so
/// `[Prereqs / TestRequires]` becomes the path `Prereqs.TestRequires`, and `;` starts a comment
/// at a line's start or after a space: <https://metacpan.org/pod/Config::MVP::Reader::INI>. A line
/// rule cannot tell which section a line sits in, as the regex crate has no lookbehind and its
/// matches do not overlap.
fn ini(text: &str, out: &mut Out) {
    let mut section = 0;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let body = line.trim_end_matches(['\n', '\r']).trim();
        if body.is_empty() || body.starts_with([';', '#']) {
            continue;
        }
        if let Some(header) = body.strip_prefix('[').and_then(|b| b.strip_suffix(']')) {
            out.path = header.split('/').map(|p| p.trim().to_owned()).collect();
            section = out.id();
            continue;
        }
        let Some((key, value)) = body.split_once('=') else {
            continue;
        };
        let value = value.find(" ;").map_or(value, |end| &value[..end]).trim();
        if value.is_empty() {
            continue;
        }
        let after = line.find('=').map_or(0, |eq| eq + 1);
        let at = start + after + line[after..].find(value).unwrap_or(0);
        out.path.push(key.trim().to_owned());
        out.leaf(value, at..at + value.len(), section);
        out.path.pop();
    }
}

/// Elements become path segments, attributes `@name` children, and text the element's value.
fn xml(text: &str, out: &mut Out) {
    let tags = regex(
        r#"(?s)<!--.*?-->|<!\[CDATA\[.*?\]\]>|<[?!][^>]*>|<(?P<close>/)?(?P<name>[\w:.-]+)(?P<attrs>(?:[^>"']|"[^"]*"|'[^']*')*?)(?P<empty>/)?>"#,
    );
    let attrs = regex(r#"(?P<key>[\w:.-]+)\s*=\s*(?:"(?P<dq>[^"]*)"|'(?P<sq>[^']*)')"#);
    let mut ids: Vec<usize> = Vec::new();
    let mut last = 0;
    for caps in tags.captures_iter(text) {
        let Some(whole) = caps.get_match() else {
            continue;
        };
        let Some(name) = caps.get_group_by_name("name") else {
            last = whole.end();
            continue;
        };
        if caps.get_group_by_name("close").is_some() {
            let raw = &text[last..whole.start()];
            let trimmed = raw.trim();
            if !trimmed.is_empty() && out.path.last().is_some_and(|n| *n == text[name.range()]) {
                let begin = last + (raw.len() - raw.trim_start().len());
                let parent = ids
                    .len()
                    .checked_sub(2)
                    .and_then(|i| ids.get(i))
                    .copied()
                    .unwrap_or(0);
                out.leaf(trimmed, begin..begin + trimmed.len(), parent);
            }
            out.path.pop();
            ids.pop();
        } else {
            out.path.push(text[name.range()].to_owned());
            let id = out.id();
            ids.push(id);
            if let Some(span) = caps.get_group_by_name("attrs") {
                for attr in attrs.captures_iter(&text[span.range()]) {
                    let (Some(key), Some(value)) = (
                        attr.get_group_by_name("key"),
                        attr.get_group_by_name("dq")
                            .or_else(|| attr.get_group_by_name("sq")),
                    ) else {
                        continue;
                    };
                    out.path.push(format!(
                        "@{}",
                        &text[span.start + key.start..span.start + key.end]
                    ));
                    let range = span.start + value.start..span.start + value.end;
                    out.leaf(&text[range.clone()], range, id);
                    out.path.pop();
                }
            }
            if caps.get_group_by_name("empty").is_some() {
                out.path.pop();
                ids.pop();
            }
        }
        last = whole.end();
    }
}

mod yaml;

use yaml::yaml;

#[cfg(test)]
mod tests;
