use serde::{Deserialize, Serialize};

use crate::Range;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A replacement of one range of document text with new text.
pub struct TextEdit {
    /// Range of text to replace.
    pub range: Range,
    /// Text that replaces the range.
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Identity of a document at the moment an edit plan was computed, used to detect stale plans.
pub struct DocumentSnapshot {
    /// URI of the document.
    pub uri: String,
    /// Editor-supplied document version, if the editor provides one.
    pub version: Option<u64>,
    /// Hash of the document text, as produced by [`document_text_hash`].
    pub text_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The text edits to apply to a single document.
pub struct DocumentEditPlan {
    /// Snapshot of the document the edits were planned against.
    pub document: DocumentSnapshot,
    /// Edits to apply to the document.
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The edit plans for every document touched by one workspace-wide change.
pub struct WorkspaceEditPlan {
    /// Per-document edit plans.
    pub documents: Vec<DocumentEditPlan>,
}

/// Hash `text` into a 16-digit lowercase hex string for stale-plan detection; not cryptographic.
pub fn document_text_hash(text: &str) -> String {
    // Stable, dependency-free FNV-1a is sufficient for stale-plan detection;
    // the document version remains the stronger check when an editor supplies it.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{hash:016x}")
}
