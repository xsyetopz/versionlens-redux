use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A zero-based line and character position in a text document.
pub struct Position {
    /// Zero-based line number.
    pub line: u32,
    /// Zero-based character offset within the line.
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A span of text between two positions in a document.
pub struct Range {
    /// Position where the range begins.
    pub start: Position,
    /// Position where the range ends.
    pub end: Position,
}
