//! Block-style YAML read by indentation, with the flow collections a value may hold.

use std::ops::Range;

use super::{Leaf, Out, unquote};

struct Frame {
    indent: usize,
    id: usize,
    item: bool,
    items: usize,
    depth: usize,
}

/// Block-style YAML by indentation. Tolerates the half-typed documents an editor sends, which a
/// strict parser would reject wholesale; block scalars are skipped.
pub(super) fn yaml(text: &str, out: &mut Out) {
    let mut stack: Vec<Frame> = Vec::new();
    let mut block: Option<usize> = None;
    let mut offset = 0;
    // Where the flow collection a line opened ends, which may be on a later line.
    let mut resume = 0;
    // A key with no value whose next line holds nothing below it is null, as pubspec's bare
    // `http:` asks for any version (dart.dev/tools/pub/dependencies), so it becomes an empty leaf.
    let mut null: Option<(Leaf, usize)> = None;
    for line in text.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        if start < resume {
            continue;
        }
        let body = line.trim_end_matches(['\n', '\r']);
        let indent = body.len() - body.trim_start_matches(' ').len();
        let content = &body[indent..];
        if content.is_empty() || content.starts_with('#') || content.starts_with("---") {
            continue;
        }
        if let Some((leaf, level)) = null.take()
            && !(indent > level || (indent == level && content.starts_with('-')))
        {
            out.leaves.push(leaf);
        }
        if let Some(level) = block {
            if indent > level {
                continue;
            }
            block = None;
        }
        let (indent, content) = items(&mut stack, out, indent, content);
        if content.is_empty() {
            continue;
        }
        let parent = stack.last().map_or(0, |f| f.id);
        let at = start + (body.len() - content.len());
        if content.starts_with(['[', '{']) {
            resume = collection(text, at, parent, out, offset);
        } else if let Some((key, value_at)) = key_value(content) {
            let raw = scalar(&content[value_at..]);
            let depth = out.path.len();
            out.path.push(key.to_owned());
            // An anchor alone, `x-common: &common`, names the mapping the lines below it hold:
            // yaml.org/spec/1.2.2/#692-node-anchors
            if raw.is_empty() || (raw.starts_with('&') && !raw.contains([' ', '\t'])) {
                if raw.is_empty() {
                    let end = at + content[..value_at].trim_end().len();
                    let leaf = Leaf {
                        path: out.path.clone(),
                        value: String::new(),
                        span: end..end,
                        parent,
                    };
                    null = Some((leaf, indent));
                }
                let id = out.id();
                stack.push(Frame {
                    indent,
                    id,
                    item: false,
                    items: 0,
                    depth,
                });
                continue;
            }
            if raw.starts_with(['|', '>']) {
                block = Some(indent);
            } else if raw.starts_with(['[', '{']) {
                resume = collection(text, at + value_at, parent, out, offset);
            } else if !raw.starts_with(['&', '*']) {
                let begin = at + value_at;
                let span = unquote(text, begin..begin + raw.len());
                out.leaf(&text[span.clone()], span, parent);
            }
            out.path.truncate(depth);
        } else {
            let raw = scalar(content);
            let span = unquote(text, at..at + raw.len());
            out.leaf(&text[span.clone()], span, parent);
        }
    }
    out.leaves.extend(null.map(|(leaf, _)| leaf));
}

/// Closes the frames a line at `indent` leaves, and opens one for each `- ` that starts it, as
/// `- - a` opens two lists. Returns where the line's content starts after them.
fn items<'a>(
    stack: &mut Vec<Frame>,
    out: &mut Out,
    mut indent: usize,
    mut content: &'a str,
) -> (usize, &'a str) {
    let mut is_item = content == "-" || content.starts_with("- ");
    loop {
        while stack
            .last()
            .is_some_and(|f| f.indent > indent || (f.indent == indent && (f.item || !is_item)))
        {
            if let Some(frame) = stack.pop() {
                out.path.truncate(frame.depth);
            }
        }
        if !is_item {
            break;
        }
        let index = stack.last_mut().map_or(0, |f| {
            f.items += 1;
            f.items - 1
        });
        let depth = out.path.len();
        out.path.push(index.to_string());
        let id = out.id();
        stack.push(Frame {
            indent,
            id,
            item: true,
            items: 0,
            depth,
        });
        let rest = content[1..].trim_start_matches(' ');
        indent += content.len() - rest.len();
        content = rest;
        is_item = content.starts_with("- ");
    }
    (indent, content)
}

/// Reads the flow collection at `at` and returns where the lines it spans end. An unclosed one, as
/// an editor sends while `{` is being typed, gives no leaves and ends with its own line at
/// `line_end`, so the lines below it still read as block YAML.
fn collection(text: &str, at: usize, parent: usize, out: &mut Out, line_end: usize) -> usize {
    let (leaves, depth) = (out.leaves.len(), out.path.len());
    flow(text, at, parent, out).unwrap_or_else(|| {
        out.leaves.truncate(leaves);
        out.path.truncate(depth);
        line_end
    })
}

/// Reads the flow collection or scalar at `at`, such as `{http: ^1.0.0, path: [a, b]}`, and
/// returns where it ends, or `None` where the document ends first. Entries may span lines and
/// carry comments, and a key with no value is null, as a block key with none is:
/// yaml.org/spec/1.2.2/#74-flow-collection-styles
fn flow(text: &str, at: usize, parent: usize, out: &mut Out) -> Option<usize> {
    let mut at = gap(text, at);
    let close = match text.as_bytes().get(at) {
        Some(b'{') => b'}',
        Some(b'[') => b']',
        _ => {
            let (span, end) = flow_scalar(text, at);
            out.leaf(&text[span.clone()], span, parent);
            return Some(end);
        }
    };
    let id = if close == b'}' { out.id() } else { parent };
    let mut index = 0;
    at += 1;
    loop {
        at = gap(text, at);
        match text.as_bytes().get(at) {
            None => return None,
            Some(&b) if b == close => return Some(at + 1),
            Some(b',') => {
                at += 1;
                continue;
            }
            _ => {}
        }
        if close == b']' {
            out.path.push(index.to_string());
            index += 1;
            at = flow(text, at, id, out)?;
        } else {
            let (key, end) = flow_scalar(text, at);
            out.path.push(text[key].to_owned());
            at = gap(text, end);
            match text[at..].strip_prefix(':') {
                Some(_) => at = flow(text, at + 1, id, out)?,
                None => out.leaf("", end..end, id),
            }
        }
        out.path.pop();
    }
}

/// Skips the spaces, line breaks, and comments between flow tokens.
fn gap(text: &str, mut at: usize) -> usize {
    loop {
        let rest = &text[at..];
        let trimmed = rest.trim_start();
        at += rest.len() - trimmed.len();
        if !trimmed.starts_with('#') {
            return at;
        }
        at += trimmed.find('\n').unwrap_or(trimmed.len());
    }
}

/// The span of the flow scalar at `at`, inside its quotes or a plain one up to the `,`, bracket,
/// `: `, or comment that ends it, and where it ends with its closing quote.
fn flow_scalar(text: &str, at: usize) -> (Range<usize>, usize) {
    let rest = &text[at..];
    if let Some(quote) = rest.chars().next().filter(|c| *c == '"' || *c == '\'') {
        let close = rest[1..].find(quote).map_or(rest.len(), |i| i + 1);
        return (at + 1..at + close, at + (close + 1).min(rest.len()));
    }
    let bytes = rest.as_bytes();
    let end = (0..bytes.len())
        .find(|&i| match bytes[i] {
            b',' | b'[' | b']' | b'{' | b'}' | b'\n' => true,
            b':' => bytes.get(i + 1).is_none_or(|n| b" \t\r\n,]}".contains(n)),
            b'#' => i > 0 && bytes[i - 1].is_ascii_whitespace(),
            _ => false,
        })
        .unwrap_or(bytes.len());
    (at..at + rest[..end].trim_end().len(), at + end)
}

/// Splits `key: value`, returning the key and the byte offset of the value within `content`.
fn key_value(content: &str) -> Option<(&str, usize)> {
    let (key, after) =
        if let Some(quote) = content.chars().next().filter(|c| *c == '"' || *c == '\'') {
            let close = content[1..].find(quote)? + 1;
            (&content[1..close], close + 1)
        } else {
            let colon = content
                .find(": ")
                .or_else(|| content.strip_suffix(':').map(str::len))?;
            (&content[..colon], colon)
        };
    let rest = content[after..].strip_prefix(':')?;
    let value_at = content.len() - rest.trim_start().len();
    Some((key.trim(), value_at))
}

/// The scalar text at the start of `raw`, without a trailing comment or whitespace.
fn scalar(raw: &str) -> &str {
    if let Some(quote) = raw.chars().next().filter(|c| *c == '"' || *c == '\'') {
        return raw[1..].find(quote).map_or(raw, |end| &raw[..end + 2]);
    }
    // A `#` after the `: ` before a value starts a comment: yaml.org/spec/1.2.2/#66-comments
    if raw.starts_with('#') {
        return "";
    }
    raw.find(" #").map_or(raw, |end| &raw[..end]).trim_end()
}
