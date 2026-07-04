use versionlens_parsers::Dependency;

mod eligibility;
mod lines;
mod text;

pub use eligibility::can_sort_dependencies;
pub(super) use text::{compare_dependencies, dependency_group, slot_end_text, slot_text_for};

pub(in crate::sort) use eligibility::is_sortable_dependency;
use lines::{dependency_start_line, sort_slot_end, sort_slot_start};

pub(super) struct SortSlot<'a> {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) line: usize,
    pub(super) dependency: &'a Dependency,
}

pub(super) fn sortable_slots<'a>(
    lines: &[&str],
    dependencies: &'a [Dependency],
) -> Vec<SortSlot<'a>> {
    dependencies
        .iter()
        .filter(|dependency| is_sortable_dependency(dependency))
        .filter_map(|dependency| {
            let line = usize::try_from(dependency_start_line(dependency)).ok()?;
            (line < lines.len()).then(|| SortSlot {
                start: sort_slot_start(lines, line, dependency),
                end: sort_slot_end(lines, line, dependency),
                line,
                dependency,
            })
        })
        .collect()
}
