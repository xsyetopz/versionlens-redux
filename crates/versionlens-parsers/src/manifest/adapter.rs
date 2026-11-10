//! One span-preserving adapter per file format. Each adapter exposes the
//! parsed document as a [`PathNode`](super::path_query::PathNode) tree whose
//! nodes keep the byte spans of the source, so path queries and the parsers
//! built on them report edit ranges against the original text.

use std::ops::Range;

pub(crate) mod json;
pub(crate) mod toml;
pub(crate) mod xml;
pub(crate) mod yaml;

/// A string that a query reached: its decoded value and the byte span of its
/// content, which excludes any quotes.
pub(crate) struct Scalar {
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}

/// The span of the lines that hold `span`, from the start of the first line
/// to the end of the last one, without its line break.
pub(crate) fn line_span(text: &str, span: &Range<usize>) -> Range<usize> {
    let start = text[..span.start].rfind('\n').map_or(0, |index| index + 1);
    let end = text[span.end..]
        .find('\n')
        .map_or(text.len(), |index| span.end + index);
    let end = if text[..end].ends_with('\r') {
        end - 1
    } else {
        end
    };
    start..end
}
