use versionlens_model::Dependency;

use super::dependency_end_line;
use super::whitespace::leading_whitespace_len;
use versionlens_model::Ecosystem::{Maven, Pub};

pub(in crate::sort::slots) fn sort_slot_start(
    lines: &[&str],
    line: usize,
    dependency: &Dependency,
) -> usize {
    if dependency.ecosystem != Pub {
        return line;
    }

    let mut start = line;
    while start > 0 && lines[start - 1].trim_start().starts_with('#') {
        start -= 1;
    }
    start
}

pub(in crate::sort::slots) fn sort_slot_end(
    lines: &[&str],
    line: usize,
    dependency: &Dependency,
) -> usize {
    if dependency.ecosystem != Pub {
        return non_pub_sort_slot_end(line, dependency);
    }

    let indent = leading_whitespace_len(lines[line]);
    let mut end = line;
    while end + 1 < lines.len()
        && !lines[end + 1].trim().is_empty()
        && leading_whitespace_len(lines[end + 1]) > indent
    {
        end += 1;
    }
    end
}

fn non_pub_sort_slot_end(line: usize, dependency: &Dependency) -> usize {
    if dependency.ecosystem == Maven {
        return usize::try_from(dependency_end_line(dependency)).unwrap_or(line);
    }

    line
}
