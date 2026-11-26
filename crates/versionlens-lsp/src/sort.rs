//! Alphabetical sorting of dependency lists. A sortable run is a block of consecutive lines that
//! each hold exactly one dependency, so comments, nested tables, and section headers stay where
//! they are and only plain one-line entries move.

use std::ops::Range;

use crate::extract::Dep;

/// One run of dependency lines: the byte range it covers and its lines sorted by name.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Run {
    pub(crate) span: Range<usize>,
    pub(crate) sorted: String,
}

/// The runs of `deps` in `text` with at least two entries. `commas` re-separates JSON entries, so
/// the entry that ends up last keeps the original last entry's (absent) trailing comma.
pub(crate) fn runs(text: &str, deps: &[Dep], commas: bool) -> Vec<Run> {
    let starts: Vec<usize> = std::iter::once(0)
        .chain(text.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let line_of = |offset: usize| starts.partition_point(|s| *s <= offset).saturating_sub(1);
    // (line, name) for each dependency alone on its line.
    let mut lines: Vec<(usize, &str)> = Vec::new();
    for dep in deps {
        let line = line_of(dep.span.start);
        if line != line_of(dep.span.end.saturating_sub(1).max(dep.span.start)) {
            continue;
        }
        match lines.iter().position(|(l, _)| *l == line) {
            Some(at) => {
                lines.remove(at);
            }
            None => lines.push((line, &dep.name)),
        }
    }
    lines.sort_unstable_by_key(|(line, _)| *line);
    let mut out = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        let mut end = at + 1;
        while end < lines.len() && lines[end].0 == lines[end - 1].0 + 1 {
            end += 1;
        }
        if end - at >= 2 {
            out.push(run(text, &starts, &lines[at..end], commas));
        }
        at = end;
    }
    out
}

fn run(text: &str, starts: &[usize], lines: &[(usize, &str)], commas: bool) -> Run {
    let body = |line: usize| {
        let end = starts.get(line + 1).map_or(text.len(), |s| s - 1);
        text[starts[line]..end].trim_end_matches('\r')
    };
    let first = starts[lines[0].0];
    let last = lines[lines.len() - 1].0;
    let span = first..starts[last] + body(last).len();
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let last_comma = body(last).trim_end().ends_with(',');
    let mut entries: Vec<(&str, String)> = lines
        .iter()
        .map(|(line, name)| {
            let text = body(*line);
            let text = if commas {
                text.trim_end().trim_end_matches(',').to_owned()
            } else {
                text.to_owned()
            };
            (*name, text)
        })
        .collect();
    entries.sort_by_cached_key(|(name, _)| name.to_lowercase());
    let count = entries.len();
    let sorted = entries
        .into_iter()
        .enumerate()
        .map(|(i, (_, line))| {
            if commas && (i + 1 < count || last_comma) {
                line + ","
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join(newline);
    Run { span, sorted }
}

#[cfg(test)]
mod tests;
