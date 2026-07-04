use versionlens_parsers::Dependency;
use versionlens_vscode_model::{Position, Range, TextEdit};

use std::ops::Range as ByteRange;

use crate::range::line_range;

mod comma;
mod groups;
mod slots;

use comma::match_trailing_comma;
use groups::grouped_slots;
pub use slots::can_sort_dependencies;
use slots::{SortSlot, compare_dependencies, slot_end_text, slot_text_for, sortable_slots};

pub fn sort_dependency_edits(text: &str, dependencies: &[Dependency]) -> Vec<TextEdit> {
    let same_line_edits = same_line_sort_edits(text, dependencies);
    if !same_line_edits.is_empty() {
        return same_line_edits;
    }

    if !can_sort_dependencies(dependencies) {
        return Vec::new();
    }

    let lines: Vec<&str> = text.lines().collect();
    let line_spans = line_body_spans(text, &lines);
    let mut slots = sortable_slots(&lines, dependencies);
    slots.sort_by_key(|slot| slot.line);

    grouped_slots(&lines, slots)
        .into_iter()
        .flat_map(|(_, group_slots)| sort_group_edits(text, &line_spans, &lines, group_slots))
        .collect()
}

fn same_line_sort_edits(text: &str, dependencies: &[Dependency]) -> Vec<TextEdit> {
    let lines: Vec<&str> = text.lines().collect();
    let mut edits = Vec::new();

    for group in same_line_groups(dependencies) {
        let Some(line) = lines.get(group.line).copied() else {
            continue;
        };
        let entries = group
            .dependencies
            .iter()
            .filter_map(|dependency| same_line_entry(line, dependency))
            .collect::<Vec<_>>();
        if entries.len() != group.dependencies.len() {
            continue;
        }

        let mut sorted = (0..group.dependencies.len()).collect::<Vec<_>>();
        sorted.sort_by(|left, right| {
            compare_dependencies(group.dependencies[*left], group.dependencies[*right])
        });
        if sorted
            .iter()
            .enumerate()
            .all(|(index, sorted_index)| index == *sorted_index)
        {
            continue;
        }

        for (target_index, source_index) in sorted.into_iter().enumerate() {
            let target = &entries[target_index];
            let source = &entries[source_index];
            if target.text == source.text {
                continue;
            }
            edits.push(TextEdit {
                range: Range {
                    start: Position {
                        line: to_u32(group.line),
                        character: to_u32(utf16_code_units(&line[..target.start])),
                    },
                    end: Position {
                        line: to_u32(group.line),
                        character: to_u32(utf16_code_units(&line[..target.end])),
                    },
                },
                new_text: source.text.to_owned(),
            });
        }
    }

    edits
}

struct SameLineGroup<'a> {
    line: usize,
    dependencies: Vec<&'a Dependency>,
}

fn same_line_groups(dependencies: &[Dependency]) -> Vec<SameLineGroup<'_>> {
    let mut groups = Vec::new();
    for dependency in dependencies
        .iter()
        .filter(|dependency| slots::is_sortable_dependency(dependency))
    {
        if dependency.range.start.line != dependency.range.end.line {
            continue;
        }
        let Some(line) = usize::try_from(dependency.range.start.line).ok() else {
            continue;
        };
        if let Some(group) = groups.iter_mut().find(|group: &&mut SameLineGroup<'_>| {
            group.line == line
                && group
                    .dependencies
                    .first()
                    .is_some_and(|existing| existing.group == dependency.group)
        }) {
            group.dependencies.push(dependency);
            continue;
        }
        groups.push(SameLineGroup {
            line,
            dependencies: vec![dependency],
        });
    }

    groups
        .into_iter()
        .filter(|group| group.dependencies.len() > 1)
        .collect()
}

struct SameLineEntry<'a> {
    start: usize,
    end: usize,
    text: &'a str,
}

fn same_line_entry<'a>(line: &'a str, dependency: &Dependency) -> Option<SameLineEntry<'a>> {
    let name_start = utf16_character_to_byte(line, dependency.range.start.character)?;
    let name_end = utf16_character_to_byte(line, dependency.range.end.character)?;
    if name_start >= name_end || name_end > line.len() {
        return None;
    }

    let start = same_line_entry_start(line, name_start);
    let end = same_line_entry_end(line, name_end);
    (start < end).then_some(SameLineEntry {
        start,
        end,
        text: line.get(start..end)?,
    })
}

fn same_line_entry_start(line: &str, name_start: usize) -> usize {
    let delimiter = line[..name_start]
        .rmatch_indices([',', '{', '['])
        .next()
        .map(|(index, delimiter)| index + delimiter.len())
        .unwrap_or(0);
    delimiter + leading_ascii_whitespace_len(&line[delimiter..name_start])
}

fn same_line_entry_end(line: &str, name_end: usize) -> usize {
    let delimiter = line[name_end..]
        .find([',', '}', ']'])
        .map(|index| name_end + index)
        .unwrap_or(line.len());
    delimiter - trailing_ascii_whitespace_len(&line[name_end..delimiter])
}

fn leading_ascii_whitespace_len(value: &str) -> usize {
    value.bytes().take_while(u8::is_ascii_whitespace).count()
}

fn trailing_ascii_whitespace_len(value: &str) -> usize {
    value
        .bytes()
        .rev()
        .take_while(u8::is_ascii_whitespace)
        .count()
}

fn utf16_character_to_byte(line: &str, character: u32) -> Option<usize> {
    let target = usize::try_from(character).ok()?;
    let mut units = 0;
    for (offset, value) in line.char_indices() {
        if units >= target {
            return Some(offset);
        }
        units += value.len_utf16();
    }
    (units == target).then_some(line.len())
}

fn utf16_code_units(value: &str) -> usize {
    value.chars().map(char::len_utf16).sum()
}

fn sort_group_edits(
    text: &str,
    line_spans: &[ByteRange<usize>],
    lines: &[&str],
    group_slots: Vec<SortSlot<'_>>,
) -> Vec<TextEdit> {
    let mut sorted: Vec<usize> = (0..group_slots.len()).collect();
    sorted.sort_by(|left, right| {
        compare_dependencies(
            group_slots[*left].dependency,
            group_slots[*right].dependency,
        )
    });
    let sorted_text: Vec<String> = sorted
        .into_iter()
        .map(|index| slot_text_for(text, line_spans, &group_slots[index]))
        .collect();

    group_slots
        .into_iter()
        .zip(sorted_text)
        .filter_map(|(slot, new_text)| {
            let current_text = slot_text_for(text, line_spans, &slot);
            let new_text = match_trailing_comma(&current_text, &new_text);
            (current_text != new_text).then(|| TextEdit {
                range: line_range(slot.start, slot.end, slot_end_text(lines, &slot)),
                new_text,
            })
        })
        .collect()
}

fn line_body_spans(text: &str, lines: &[&str]) -> Vec<ByteRange<usize>> {
    let mut spans = Vec::with_capacity(lines.len());
    let mut offset = 0;

    for line in lines {
        let start = offset;
        let end = start + line.len();
        spans.push(start..end);
        offset = end + line_ending_len(text.get(end..).unwrap_or_default());
    }

    spans
}

fn line_ending_len(text: &str) -> usize {
    if text.starts_with("\r\n") {
        2
    } else {
        usize::from(text.starts_with('\n') || text.starts_with('\r'))
    }
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests;
