use crate::binding::NativeDocumentInput;

pub(super) fn document_weight(input: &NativeDocumentInput) -> usize {
    input
        .text
        .capacity()
        .saturating_add(input.uri.capacity())
        .saturating_add(input.language_id.capacity())
        .saturating_add(input.workspace_root.as_ref().map_or(0, String::capacity))
}
