use crate::sort::slots::SortSlot;
use Range as ByteRange;
use std::cmp::Ordering;
use std::ops::Range;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Ruby;

pub(in crate::sort) fn dependency_group(dependency: &Dependency) -> &str {
    dependency.group.as_str()
}

pub(in crate::sort) fn compare_dependencies(left: &Dependency, right: &Dependency) -> Ordering {
    compare_names(dependency_sort_name(left), dependency_sort_name(right))
}

fn dependency_sort_name(dependency: &Dependency) -> &str {
    if dependency.ecosystem == Ruby
        && let Some(hosted_name) = &dependency.hosted_name
    {
        return hosted_name;
    }

    &dependency.name
}

fn compare_names(left: &str, right: &str) -> Ordering {
    let left_key = locale_compare_key(left);
    let right_key = locale_compare_key(right);
    left_key
        .base
        .cmp(&right_key.base)
        .then_with(|| left_key.accents.cmp(&right_key.accents))
        .then_with(|| left_key.case_weights.cmp(&right_key.case_weights))
}

fn locale_compare_key(value: &str) -> LocaleCompareKey {
    let mut key = LocaleCompareKey {
        base: vec![],
        accents: vec![],
        case_weights: vec![],
    };
    for character in value.chars().map(locale_compare_character) {
        key.base.push(character.base);
        key.accents.push(character.accent);
        key.case_weights.push(character.case_weight);
    }
    key
}

fn locale_compare_character(value: char) -> LocaleCompareCharacter {
    let lowered = value.to_lowercase().next().unwrap_or(value);
    let decomposed: Vec<char> = lowered.nfd().collect();
    let base = decomposed
        .iter()
        .copied()
        .find(|character| !is_combining_mark(*character))
        .unwrap_or(lowered);
    let accent = decomposed
        .into_iter()
        .filter(|character| is_combining_mark(*character))
        .collect::<Vec<char>>();
    LocaleCompareCharacter {
        base,
        accent,
        case_weight: u8::from(value.is_uppercase()),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct LocaleCompareCharacter {
    base: char,
    accent: Vec<char>,
    case_weight: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleCompareKey {
    base: Vec<char>,
    accents: Vec<Vec<char>>,
    case_weights: Vec<u8>,
}

type LineSpans<'a> = &'a [ByteRange<usize>];

pub(in crate::sort) fn slot_text_for(
    text: &str,
    line_spans: LineSpans<'_>,
    slot: &SortSlot<'_>,
) -> String {
    slot_text(text, line_spans, slot.start, slot.end)
}

pub(in crate::sort) fn slot_end_text<'a>(lines: &'a [&str], slot: &SortSlot<'_>) -> &'a str {
    lines[slot.end]
}

fn slot_text(text: &str, line_spans: LineSpans<'_>, start: usize, end: usize) -> String {
    let Some(start_offset) = line_spans.get(start).map(|span| span.start) else {
        return "".to_owned();
    };
    let Some(end_offset) = line_spans.get(end).map(|span| span.end) else {
        return "".to_owned();
    };

    text.get(start_offset..end_offset)
        .unwrap_or_default()
        .to_owned()
}
