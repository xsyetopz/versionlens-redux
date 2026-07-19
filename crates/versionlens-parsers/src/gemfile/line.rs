use crate::positions::line_range;
use versionlens_model::Range;

pub(super) struct GemLineContext<'a> {
    pub(super) line_index: usize,
    pub(super) line: &'a str,
    pub(super) offset: usize,
    pub(super) content: &'a str,
    pub(super) group: &'a str,
}

pub(super) struct GemNameSpan<'a> {
    pub(super) name: &'a str,
    pub(super) end: usize,
}

pub(super) fn gem_name_range(context: &GemLineContext<'_>) -> Range {
    line_range(
        context.line_index,
        context.line,
        context.offset,
        context.offset,
    )
}
