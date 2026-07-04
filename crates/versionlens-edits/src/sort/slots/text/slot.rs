use std::ops::Range as ByteRange;

use crate::sort::slots::SortSlot;

pub(in crate::sort) fn slot_text_for(
    text: &str,
    line_spans: &[ByteRange<usize>],
    slot: &SortSlot<'_>,
) -> String {
    slot_text(text, line_spans, slot.start, slot.end)
}

pub(in crate::sort) fn slot_end_text<'a>(lines: &'a [&str], slot: &SortSlot<'_>) -> &'a str {
    lines[slot.end]
}

fn slot_text(text: &str, line_spans: &[ByteRange<usize>], start: usize, end: usize) -> String {
    let Some(start_offset) = line_spans.get(start).map(|span| span.start) else {
        return String::new();
    };
    let Some(end_offset) = line_spans.get(end).map(|span| span.end) else {
        return String::new();
    };

    text.get(start_offset..end_offset)
        .unwrap_or_default()
        .to_owned()
}
