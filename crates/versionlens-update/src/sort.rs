use self::slots::is_sortable_dependency;
use std::ops::Range as ByteRange;

use versionlens_model::Dependency;
use versionlens_model::{Position, Range, TextEdit};

use crate::range::line_range;

pub(crate) type SortTextEdits = Vec<TextEdit>;

mod comma;
mod groups;
mod slots;

use comma::match_trailing_comma;
use groups::grouped_slots;
pub use slots::can_sort_dependencies;
use slots::{SortSlot, compare_dependencies, slot_end_text, slot_text_for, sortable_slots};

/// The edits that sort `dependencies` in `text`, empty when they cannot be
/// sorted safely.
pub fn sort_dependency_edits(text: &str, dependencies: &[Dependency]) -> SortTextEdits {
    let same_line_edits = same_line_sort_edits(text, dependencies);
    if !same_line_edits.is_empty() {
        return same_line_edits;
    }

    if !can_sort_dependencies(dependencies) {
        return vec![];
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

fn same_line_sort_edits(text: &str, dependencies: &[Dependency]) -> SortTextEdits {
    let lines: Vec<&str> = text.lines().collect();
    let mut edits = vec![];

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
    let mut groups = vec![];
    for dependency in dependencies
        .iter()
        .filter(|dependency| is_sortable_dependency(dependency))
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

#[derive(Clone, Copy)]
struct StructuralDelimiter {
    index: usize,
    delimiter: char,
    depth_before: usize,
}

fn structural_delimiters(line: &str) -> Vec<StructuralDelimiter> {
    let mut delimiters = Vec::new();
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for (index, value) in line.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if value == '\\' {
                escaped = true;
            } else if value == active_quote {
                quote = None;
            }
            continue;
        }

        if value == '"' || value == '\'' {
            quote = Some(value);
            continue;
        }

        let depth_before = depth;
        match value {
            '{' | '[' => {
                delimiters.push(StructuralDelimiter {
                    index,
                    delimiter: value,
                    depth_before,
                });
                depth += 1;
            }
            '}' | ']' => {
                delimiters.push(StructuralDelimiter {
                    index,
                    delimiter: value,
                    depth_before,
                });
                depth = depth.saturating_sub(1);
            }
            ',' => delimiters.push(StructuralDelimiter {
                index,
                delimiter: value,
                depth_before,
            }),
            _ => {}
        }
    }
    delimiters
}

fn depth_at(line: &str, offset: usize) -> usize {
    let mut depth = 0usize;
    for delimiter in structural_delimiters(line) {
        if delimiter.index >= offset {
            break;
        }
        match delimiter.delimiter {
            '{' | '[' => depth += 1,
            '}' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    depth
}

fn same_line_entry_start(line: &str, name_start: usize) -> usize {
    let depth = depth_at(line, name_start);
    let delimiter = structural_delimiters(line)
        .into_iter()
        .rfind(|delimiter| {
            delimiter.index < name_start
                && ((delimiter.delimiter == ',' && delimiter.depth_before == depth)
                    || ((delimiter.delimiter == '{' || delimiter.delimiter == '[')
                        && delimiter.depth_before + 1 == depth))
        })
        .map_or(0, |delimiter| {
            delimiter.index + delimiter.delimiter.len_utf8()
        });
    delimiter + leading_ascii_whitespace_len(&line[delimiter..name_start])
}

fn same_line_entry_end(line: &str, name_end: usize) -> usize {
    let depth = depth_at(line, name_end);
    let delimiter = structural_delimiters(line)
        .into_iter()
        .find(|delimiter| {
            delimiter.index >= name_end
                && delimiter.depth_before == depth
                && matches!(delimiter.delimiter, ',' | '}' | ']')
        })
        .map_or(line.len(), |delimiter| delimiter.index);
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
) -> SortTextEdits {
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
    let mut spans = vec![];
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
mod tests {
    use std::cmp::Reverse;
    use versionlens_model::{Dependency, Ecosystem, Position, Range, TextEdit};

    use super::{can_sort_dependencies, sort_dependency_edits};
    use crate::support::tests::range;
    use versionlens_model::Ecosystem::*;

    #[test]
    fn sorts_requirements_dependency_lines_by_package_name() {
        let text = package_file_fixture("sorts-requirements-dependency-lines-by-package-name.txt");
        let dependencies = vec![
            dependency("zeta", range(0, 0, 0, 7)),
            dependency("alpha", range(2, 0, 2, 8)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(can_sort_dependencies(&dependencies));
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].range, range(0, 0, 0, 7));
        assert_eq!(edits[0].new_text, "alpha==1");
        assert_eq!(edits[1].range, range(2, 0, 2, 8));
        assert_eq!(edits[1].new_text, "zeta==1");
    }

    #[test]
    fn sort_edit_line_ranges_count_utf16_code_units() {
        let text = package_file_fixture("sort-edit-line-ranges-count-utf16-code-units.txt");
        let dependencies = vec![
            dependency("zeta😀", range(0, 0, 0, 6)),
            dependency("alpha", range(1, 0, 1, 8)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].range, range(0, 0, 0, 9));
        assert_eq!(edits[0].new_text, "alpha==1");
        assert_eq!(edits[1].range, range(1, 0, 1, 8));
        assert_eq!(edits[1].new_text, "zeta😀==1");
    }

    #[test]
    fn sorts_dependency_names_like_upstream_locale_compare() {
        let text = package_file_fixture("sorts-dependency-names-like-upstream-locale-compare.txt");
        let dependencies = vec![
            dependency("Alpha", range(0, 0, 0, 8)),
            dependency("alpha", range(1, 0, 1, 8)),
            dependency("Beta", range(2, 0, 2, 7)),
            dependency("beta", range(3, 0, 3, 7)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_sorted_prefix(&edits, "Alpha==1");
        assert_eq!(edits[2].new_text, "beta==1");
        assert_eq!(edits[3].new_text, "Beta==1");
    }

    #[test]
    fn sorts_accented_dependency_names_like_upstream_locale_compare() {
        let text = package_file_fixture(
            "sorts-accented-dependency-names-like-upstream-locale-compare.txt",
        );
        let dependencies = vec![
            dependency("alpha", range(0, 0, 0, 8)),
            dependency("ä", range(1, 0, 1, 5)),
            dependency("Á", range(2, 0, 2, 5)),
            dependency("a", range(3, 0, 3, 4)),
            dependency("Ä", range(4, 0, 4, 5)),
            dependency("á", range(5, 0, 5, 5)),
            dependency("A", range(6, 0, 6, 4)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 7);
        assert_eq!(edits[0].new_text, "a==1");
        assert_eq!(edits[1].new_text, "A==1");
        assert_eq!(edits[2].new_text, "á==1");
        assert_eq!(edits[3].new_text, "Á==1");
        assert_eq!(edits[4].new_text, "ä==1");
        assert_eq!(edits[5].new_text, "Ä==1");
        assert_eq!(edits[6].new_text, "alpha==1");
    }

    #[test]
    fn sorts_pyproject_python_dependency_groups() {
        let text = package_file_fixture("sorts-pyproject-python-dependency-groups.txt");
        let dependencies = vec![
            dependency_with(Python, "project.dependencies", "zeta", range(0, 1, 0, 5)),
            dependency_with(Python, "project.dependencies", "alpha", range(1, 1, 1, 6)),
            dependency_with(
                Python,
                "project.optional-dependencies.test",
                "z-optional",
                range(2, 1, 2, 11),
            ),
            dependency_with(
                Python,
                "project.optional-dependencies.test",
                "a-optional",
                range(3, 1, 3, 11),
            ),
            dependency_with(
                Python,
                "tool.poetry.dependencies",
                "zeta",
                range(4, 0, 4, 4),
            ),
            dependency_with(
                Python,
                "tool.poetry.dependencies",
                "alpha",
                range(5, 0, 5, 5),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 6);
        assert_eq!(edits[0].new_text, "\"alpha==1\"");
        assert_eq!(edits[1].new_text, "\"zeta==1\"");
        assert_eq!(edits[2].new_text, "\"a-optional==1\"");
        assert_eq!(edits[3].new_text, "\"z-optional==1\"");
        assert_eq!(edits[4].new_text, "alpha = \"1\"");
        assert_eq!(edits[5].new_text, "zeta = \"1\"");
    }

    #[test]
    fn sorts_each_dependency_group_independently() {
        let text = package_file_fixture("sorts-each-dependency-group-independently.txt");
        let dependencies = vec![
            dependency_with(Pub, "dependencies", "zeta", range(0, 0, 0, 7)),
            dependency_with(Pub, "dependencies", "alpha", range(1, 0, 1, 8)),
            dependency_with(Pub, "dev_dependencies", "z-dev", range(2, 0, 2, 8)),
            dependency_with(Pub, "dev_dependencies", "a-dev", range(3, 0, 3, 8)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_sorted_prefix(&edits, "zeta==1");
        assert_eq!(edits[2].new_text, "a-dev==1");
        assert_eq!(edits[3].new_text, "z-dev==1");
    }

    #[test]
    fn sorts_pub_dependencies_with_leading_comments() {
        let text = package_file_fixture("sorts-pub-dependencies-with-leading-comments.txt");
        let dependencies = pub_comment_dependencies();

        let edits = sort_dependency_edits(text, &dependencies);

        assert_comment_sort(&edits, "  # alpha\n  alpha: 1", "  # zeta\n  zeta: 1");
    }

    #[test]
    fn sorts_multiline_slots_preserving_original_line_endings() {
        let text =
            package_file_fixture("sorts-multiline-slots-preserving-original-line-endings.txt");
        let dependencies = vec![
            dependency_with(Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
            dependency_with(Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "  # alpha\r\n  alpha: 1");
        assert_eq!(edits[1].new_text, "  # zeta\r\n  zeta: 1");
    }

    #[test]
    fn sorts_pub_dependencies_with_inline_comments() {
        let text = package_file_fixture("sorts-pub-dependencies-with-inline-comments.txt");
        let dependencies = vec![
            dependency_with(Pub, "dependencies", "zeta", range(0, 2, 0, 6)),
            dependency_with(Pub, "dependencies", "alpha", range(1, 2, 1, 7)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "  alpha: 1 # first");
        assert_eq!(edits[1].new_text, "  zeta: 1 # zed");
    }

    #[test]
    fn sorts_pub_dependencies_with_mixed_comments() {
        let text = package_file_fixture("sorts-pub-dependencies-with-mixed-comments.txt");
        let dependencies = vec![
            dependency_with(Pub, "dependencies", "http", range(0, 2, 0, 6)),
            dependency_with(Pub, "dependencies", "glob", range(1, 2, 1, 6)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "  glob: # version child property\n    version: '1.2.*'"
        );
        assert_eq!(edits[1].new_text, "  http: # blank entry with comment");
    }

    #[test]
    fn sorts_json_style_dependencies_that_share_a_line() {
        let text = package_file_fixture("sorts-json-style-dependencies-that-share-a-line.txt");
        let dependencies = vec![
            dependency_with(Npm, "dependencies", "zeta", range(0, 17, 0, 23)),
            dependency_with(Npm, "dependencies", "alpha", range(0, 28, 0, 35)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].range, range(0, 17, 0, 27));
        assert_eq!(edits[0].new_text, r#""alpha":"1""#);
        assert_eq!(edits[1].range, range(0, 28, 0, 39));
        assert_eq!(edits[1].new_text, r#""zeta":"1""#);
    }

    #[test]
    fn same_line_sort_ignores_quoted_and_nested_delimiters() {
        let text = r#"{"dependencies":{"zeta":{"description":"contains , } ] and \"quote\"","path":"C:\\tmp\\[x],{y}","nested":[1,2]},"alpha":{"description":"a,b","nested":{"x":"[braces]"}}}}"#;
        let zeta = u32::try_from(text.find("\"zeta\"").unwrap()).unwrap();
        let alpha = u32::try_from(text.find("\"alpha\"").unwrap()).unwrap();
        let dependencies = vec![
            dependency_with(Npm, "dependencies", "zeta", range(0, zeta, 0, zeta + 6)),
            dependency_with(Npm, "dependencies", "alpha", range(0, alpha, 0, alpha + 7)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert!(edits[0].new_text.contains(r#"description":"a,b"#));
        assert!(edits[0].new_text.contains(r#"nested":{"x":"[braces]"}"#));
        assert!(
            edits[1]
                .new_text
                .contains(r#"contains , } ] and \"quote\""#)
        );
        assert!(edits[1].new_text.contains(r#"nested":[1,2]"#));

        let updated = apply_same_line_edits(text, &edits);
        serde_json::from_str::<serde_json::Value>(&updated).expect("sorted JSON remains parseable");
    }

    #[test]
    fn preserves_json_trailing_commas_for_target_slots() {
        let text = package_file_fixture("preserves-json-trailing-commas-for-target-slots.txt");
        let dependencies = vec![
            dependency_with(Npm, "dependencies", "zeta", range(2, 4, 2, 15)),
            dependency_with(Npm, "dependencies", "alpha", range(3, 4, 3, 16)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_json_sort(&edits);
    }

    fn assert_sorted_prefix(edits: &[TextEdit], second: &str) {
        assert_eq!(edits.len(), 4);
        assert_eq!(edits[0].new_text, "alpha==1");
        assert_eq!(edits[1].new_text, second);
    }

    fn assert_comment_sort(edits: &[TextEdit], first: &str, second: &str) {
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].range, range(0, 0, 1, 9));
        assert_eq!(edits[0].new_text, first);
        assert_eq!(edits[1].range, range(2, 0, 3, 10));
        assert_eq!(edits[1].new_text, second);
    }

    fn assert_json_sort(edits: &[TextEdit]) {
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "    \"alpha\": \"1\",");
        assert_eq!(edits[1].new_text, "    \"zeta\": \"1\"");
    }

    fn pub_comment_dependencies() -> Vec<Dependency> {
        vec![
            dependency_with(Pub, "dependencies", "zeta", range(1, 2, 1, 6)),
            dependency_with(Pub, "dependencies", "alpha", range(3, 2, 3, 7)),
        ]
    }
    #[test]
    fn sorts_package_json_dependencies_when_metadata_is_parsed() {
        let text =
            package_file_fixture("sorts-package-json-dependencies-when-metadata-is-parsed.txt");
        let dependencies = vec![
            dependency_with(Npm, "version", "package", range(1, 2, 1, 21)),
            dependency_with(Npm, "packageManager", "pnpm", range(2, 2, 2, 33)),
            dependency_with(Npm, "dependencies", "zeta", range(4, 4, 4, 15)),
            dependency_with(Npm, "dependencies", "alpha", range(5, 4, 5, 16)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_json_sort(&edits);
    }

    #[test]
    fn sorts_smoke_jspm_dependencies() {
        let text = package_file_fixture("sorts-smoke-jspm-dependencies.txt");
        let dependencies = vec![
            dependency_with(Npm, "jspm.dependencies", "webpack", range(3, 6, 3, 15)),
            dependency_with(Npm, "jspm.dependencies", "bluebird", range(4, 6, 4, 16)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "      \"bluebird\": \"npm:bluebird@^3.7.2\","
        );
        assert_eq!(edits[1].new_text, "      \"webpack\": \"npm:webpack@*\"");
    }

    #[test]
    fn sorts_composer_dependencies_without_moving_manifest_version() {
        let text =
            package_file_fixture("sorts-composer-dependencies-without-moving-manifest-version.txt");
        let dependencies = vec![
            dependency_with(Composer, "version", "1.0.0", range(1, 2, 1, 21)),
            dependency_with(Composer, "require", "symfony/console", range(3, 4, 3, 21)),
            dependency_with(Composer, "require", "allocine/twigcs", range(4, 4, 4, 21)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "    \"allocine/twigcs\": \"^3.1.3\",");
        assert_eq!(edits[1].new_text, "    \"symfony/console\": \"8.1.*\"");
    }

    #[test]
    fn sorts_deno_imports_with_npm_imports() {
        let text = package_file_fixture("sorts-deno-imports-with-npm-imports.txt");
        let dependencies = vec![
            dependency_with(Deno, "imports", "zeta", range(2, 4, 2, 10)),
            dependency_with(Npm, "imports", "chalk", range(3, 4, 3, 11)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "    \"chalk\": \"npm:chalk@5.3.0\",");
        assert_eq!(edits[1].new_text, "    \"zeta\": \"jsr:@scope/zeta@1.0.0\"");
    }

    #[test]
    fn sorts_deno_scoped_imports_within_each_scope() {
        let text = package_file_fixture("sorts-deno-scoped-imports-within-each-scope.txt");
        let dependencies = vec![
            dependency_with(
                Npm,
                "scopes.https://deno.land/x/app/",
                "zeta",
                range(3, 6, 3, 12),
            ),
            dependency_with(
                Npm,
                "scopes.https://deno.land/x/app/",
                "chalk",
                range(4, 6, 4, 13),
            ),
            dependency_with(
                Deno,
                "scopes.https://deno.land/x/other/",
                "@scope/bravo",
                range(7, 6, 7, 13),
            ),
            dependency_with(
                Deno,
                "scopes.https://deno.land/x/other/",
                "@scope/alpha",
                range(8, 6, 8, 13),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 4);
        assert_eq!(edits[0].new_text, "      \"chalk\": \"npm:chalk@5.3.0\",");
        assert_eq!(edits[1].new_text, "      \"zeta\": \"npm:zeta@1.0.0\"");
        assert_eq!(
            edits[2].new_text,
            "      \"alpha\": \"jsr:@scope/alpha@1.0.0\","
        );
        assert_eq!(
            edits[3].new_text,
            "      \"bravo\": \"jsr:@scope/bravo@1.0.0\""
        );
    }

    #[test]
    fn sorts_pnpm_workspace_catalogs() {
        let text = package_file_fixture("sorts-pnpm-workspace-catalogs.txt");
        let dependencies = vec![
            dependency_with(Npm, "catalogs.react18", "react-dom", range(2, 4, 2, 13)),
            dependency_with(Npm, "catalogs.react18", "react", range(3, 4, 3, 9)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "    react: ^18.3.1");
        assert_eq!(edits[1].new_text, "    react-dom: ^19.2.7");
    }

    #[test]
    fn sorts_package_json_named_workspace_catalogs() {
        let text = package_file_fixture("sorts-package-json-named-workspace-catalogs.txt");
        let dependencies = vec![
            dependency_with(
                Npm,
                "workspaces.catalogs.react18",
                "react-dom",
                range(4, 8, 4, 19),
            ),
            dependency_with(
                Npm,
                "workspaces.catalogs.react18",
                "react",
                range(5, 8, 5, 15),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "        \"react\": \"^18.3.1\",");
        assert_eq!(edits[1].new_text, "        \"react-dom\": \"^19.2.7\"");
    }
    #[test]
    fn sorts_cargo_dependency_tables() {
        let text = package_file_fixture("sorts-cargo-dependency-tables.txt");
        let dependencies = vec![
            dependency_with(Cargo, "dev-dependencies", "syn", range(1, 0, 1, 3)),
            dependency_with(Cargo, "dev-dependencies", "axum-extra", range(2, 0, 2, 10)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "axum-extra = \"0.12\"");
        assert_eq!(edits[1].new_text, "syn = \"2\"");
    }

    #[test]
    fn sorts_cargo_renamed_dependencies_by_local_alias() {
        let text = package_file_fixture("sorts-cargo-renamed-dependencies-by-local-alias.txt");
        let dependencies = vec![
            Dependency {
                hosted_name: Some("alpha".to_owned()),
                ..dependency_with(Cargo, "dependencies", "z_alias", range(1, 0, 1, 7))
            },
            Dependency {
                hosted_name: Some("zeta".to_owned()),
                ..dependency_with(Cargo, "dependencies", "a_alias", range(2, 0, 2, 7))
            },
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "a_alias = { package = \"zeta\", version = \"1\" }"
        );
        assert_eq!(
            edits[1].new_text,
            "z_alias = { package = \"alpha\", version = \"1\" }"
        );
    }

    #[test]
    fn sorts_composer_package_link_groups() {
        let text = package_file_fixture("sorts-composer-package-link-groups.txt");
        let dependencies = vec![
            dependency_with(Composer, "conflict", "zeta/package", range(0, 0, 0, 14)),
            dependency_with(Composer, "conflict", "alpha/package", range(1, 0, 1, 15)),
            dependency_with(
                Composer,
                "replace",
                "vendor/replaced-zeta",
                range(2, 0, 2, 22),
            ),
            dependency_with(
                Composer,
                "replace",
                "vendor/replaced-alpha",
                range(3, 0, 3, 23),
            ),
            dependency_with(
                Composer,
                "provide",
                "psr/zeta-implementation",
                range(4, 0, 4, 25),
            ),
            dependency_with(
                Composer,
                "provide",
                "psr/alpha-implementation",
                range(5, 0, 5, 26),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 6);
        assert_eq!(edits[0].new_text, r#""alpha/package": "<2","#);
        assert_eq!(edits[1].new_text, r#""zeta/package": "<2","#);
        assert_eq!(
            edits[2].new_text,
            r#""vendor/replaced-alpha": "self.version","#
        );
        assert_eq!(
            edits[3].new_text,
            r#""vendor/replaced-zeta": "self.version","#
        );
        assert_eq!(edits[4].new_text, r#""psr/alpha-implementation": "1.0","#);
        assert_eq!(edits[5].new_text, r#""psr/zeta-implementation": "1.0""#);
    }

    #[test]
    fn sorts_maven_dependency_nodes() {
        let text = package_file_fixture("sorts-maven-dependency-nodes.txt");
        let dependencies = vec![
            dependency_with(
                Maven,
                "project.dependencies.dependency",
                "org.zeta:zeta",
                range(1, 2, 5, 15),
            ),
            dependency_with(
                Maven,
                "project.dependencies.dependency",
                "org.alpha:alpha",
                range(6, 2, 10, 15),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "  <dependency>\n    <groupId>org.alpha</groupId>\n    <artifactId>alpha</artifactId>\n    <version>1</version>\n  </dependency>"
        );
        assert_eq!(
            edits[1].new_text,
            "  <dependency>\n    <groupId>org.zeta</groupId>\n    <artifactId>zeta</artifactId>\n    <version>1</version>\n  </dependency>"
        );
    }

    #[test]
    fn sorts_maven_dependency_management_nodes() {
        let text = package_file_fixture("sorts-maven-dependency-management-nodes.txt");
        let dependencies = vec![
            dependency_with(
                Maven,
                "project.dependencyManagement.dependencies.dependency",
                "org.zeta:zeta",
                range(2, 4, 7, 17),
            ),
            dependency_with(
                Maven,
                "project.dependencyManagement.dependencies.dependency",
                "org.alpha:alpha",
                range(8, 4, 13, 17),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "    <dependency>\n      <groupId>org.alpha</groupId>\n      <type>pom</type>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>"
        );
        assert_eq!(
            edits[1].new_text,
            "    <dependency>\n      <groupId>org.zeta</groupId>\n      <type>pom</type>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>"
        );
    }

    #[test]
    fn sorts_go_require_block_dependencies() {
        let text = package_file_fixture("sorts-go-require-block-dependencies.txt");
        let dependencies = vec![
            dependency_with(Go, "require", "zeta.example/pkg", range(1, 1, 1, 17)),
            dependency_with(Go, "require", "alpha.example/pkg", range(2, 1, 2, 18)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "\talpha.example/pkg v1.0.0 // indirect");
        assert_eq!(edits[1].new_text, "\tzeta.example/pkg v1.0.0");
    }

    #[test]
    fn does_not_sort_go_dependencies_across_require_blocks() {
        let text = package_file_fixture("does-not-sort-go-dependencies-across-require-blocks.txt");
        let dependencies = vec![
            dependency_with(Go, "require", "zeta.example/pkg", range(1, 1, 1, 17)),
            dependency_with(Go, "require", "alpha.example/pkg", range(5, 1, 5, 18)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn sorts_ruby_gemfile_dependencies() {
        let text = package_file_fixture("sorts-ruby-gemfile-dependencies.txt");
        let dependencies = vec![
            dependency_with(Ruby, "dependencies", "zeta", range(0, 5, 0, 9)),
            dependency_with(Ruby, "dependencies", "alpha", range(1, 5, 1, 10)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
        assert_eq!(edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
    }

    #[test]
    fn sorts_ruby_github_dependencies_by_gem_name() {
        let text = package_file_fixture("sorts-ruby-github-dependencies-by-gem-name.txt");
        let dependencies = vec![
            Dependency {
                hosted_name: Some("zeta".to_owned()),
                ..dependency_with(Ruby, "dependencies", "org/zeta", range(0, 5, 0, 9))
            },
            Dependency {
                hosted_name: Some("alpha".to_owned()),
                ..dependency_with(Ruby, "dependencies", "org/alpha", range(1, 5, 1, 10))
            },
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "gem \"alpha\", github: \"org/alpha\", tag: \"v1.0.0\""
        );
        assert_eq!(
            edits[1].new_text,
            "gem \"zeta\", github: \"org/zeta\", tag: \"v1.0.0\""
        );
    }
    #[test]
    fn does_not_sort_ruby_dependencies_across_group_blocks() {
        let text = package_file_fixture("does-not-sort-ruby-dependencies-across-group-blocks.txt");
        let dependencies = vec![
            dependency_with(
                Ruby,
                "group :production",
                "zeta",
                unsupported_range(1, 7, 1, 11),
            ),
            dependency_with(
                Ruby,
                "group :production",
                "alpha",
                unsupported_range(5, 7, 5, 12),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn sorts_dotnet_single_line_package_references() {
        let text = package_file_fixture("sorts-dotnet-single-line-package-references.txt");
        let dependencies = vec![
            dependency_with(
                Dotnet,
                "PackageReference",
                "Zeta.Package",
                unsupported_range(1, 2, 1, 61),
            ),
            dependency_with(
                Dotnet,
                "PackageReference",
                "Alpha.Package",
                unsupported_range(2, 2, 2, 62),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits.len(), 2);
        assert_eq!(
            edits[0].new_text,
            "  <PackageReference Include=\"Alpha.Package\" Version=\"1\" />"
        );
        assert_eq!(
            edits[1].new_text,
            "  <PackageReference Include=\"Zeta.Package\" Version=\"1\" />"
        );
    }

    #[test]
    fn skips_dotnet_version_insertion_ranges() {
        let text = package_file_fixture("skips-dotnet-version-insertion-ranges.txt");
        let dependencies = vec![
            Dependency {
                requirement_prefix: " Version=\"".to_owned(),
                requirement_suffix: "\"".to_owned(),
                canonical_reference: None,
                ..dependency_with(
                    Dotnet,
                    "PackageReference",
                    "Zeta.Package",
                    unsupported_range(1, 28, 1, 40),
                )
            },
            Dependency {
                requirement_prefix: " Version=\"".to_owned(),
                requirement_suffix: "\"".to_owned(),
                canonical_reference: None,
                ..dependency_with(
                    Dotnet,
                    "PackageReference",
                    "Alpha.Package",
                    unsupported_range(2, 28, 2, 41),
                )
            },
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_julia_dependencies() {
        let text = package_file_fixture("skips-julia-dependencies.txt");
        let dependencies = vec![
            dependency_with(Julia, "compat", "Zeta", unsupported_range(1, 0, 1, 10)),
            dependency_with(Julia, "compat", "Alpha", unsupported_range(2, 0, 2, 11)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_cran_dependencies() {
        let text = package_file_fixture("skips-cran-dependencies.txt");
        let dependencies = vec![
            dependency_with(Cran, "Imports", "zeta", unsupported_range(0, 9, 0, 13)),
            dependency_with(Cran, "Imports", "alpha", unsupported_range(0, 24, 0, 29)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_conan_dependencies() {
        let text = package_file_fixture("skips-conan-dependencies.txt");
        let dependencies = vec![
            dependency_with(Conan, "requires", "zlib", unsupported_range(1, 0, 1, 4)),
            dependency_with(Conan, "requires", "poco", unsupported_range(2, 0, 2, 4)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_vcpkg_dependencies() {
        let text = package_file_fixture("skips-vcpkg-dependencies.txt");
        let dependencies = vec![
            dependency_with(
                Vcpkg,
                "dependencies",
                "zlib",
                unsupported_range(0, 18, 0, 22),
            ),
            dependency_with(
                Vcpkg,
                "dependencies",
                "fmt",
                unsupported_range(0, 32, 0, 35),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_swift_package_dependencies() {
        let text = package_file_fixture("skips-swift-package-dependencies.txt");
        let dependencies = vec![
            dependency_with(
                Swift,
                "dependencies",
                "zlib",
                unsupported_range(1, 60, 1, 65),
            ),
            dependency_with(
                Swift,
                "dependencies",
                "fmt",
                unsupported_range(2, 59, 2, 65),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_zig_zon_dependencies() {
        let text = package_file_fixture("skips-zig-zon-dependencies.txt");
        let dependencies = vec![
            dependency_with(Zig, "dependencies", "zlib", unsupported_range(0, 44, 0, 72)),
            dependency_with(Zig, "dependencies", "fmt", unsupported_range(0, 95, 0, 122)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_nimble_dependencies() {
        let text = package_file_fixture("skips-nimble-dependencies.txt");
        let dependencies = vec![
            dependency_with(Nim, "requires", "zlib", unsupported_range(0, 10, 0, 14)),
            dependency_with(Nim, "requires", "fmt", unsupported_range(1, 10, 1, 13)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_luarocks_rockspec_dependencies() {
        let text = package_file_fixture("skips-luarocks-rockspec-dependencies.txt");
        let dependencies = vec![
            dependency_with(
                LuaRocks,
                "dependencies",
                "zlib",
                unsupported_range(1, 4, 1, 8),
            ),
            dependency_with(
                LuaRocks,
                "dependencies",
                "busted",
                unsupported_range(2, 4, 2, 10),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_cpanfile_dependencies() {
        let text = package_file_fixture("skips-cpanfile-dependencies.txt");
        let dependencies = vec![
            dependency_with(Cpan, "requires", "Zeta", unsupported_range(0, 10, 0, 14)),
            dependency_with(Cpan, "requires", "Alpha", unsupported_range(1, 10, 1, 15)),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_haxelib_json_dependencies() {
        let text = package_file_fixture("skips-haxelib-json-dependencies.txt");
        let dependencies = vec![
            dependency_with(
                Haxelib,
                "dependencies",
                "zeta",
                unsupported_range(0, 18, 0, 22),
            ),
            dependency_with(
                Haxelib,
                "dependencies",
                "alpha",
                unsupported_range(0, 35, 0, 40),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_terraform_required_providers() {
        let text = package_file_fixture("skips-terraform-required-providers.txt");
        let dependencies = vec![
            dependency_with(
                Terraform,
                "required_providers",
                "hashicorp/zeta",
                unsupported_range(0, 33, 0, 37),
            ),
            dependency_with(
                Terraform,
                "required_providers",
                "hashicorp/alpha",
                unsupported_range(0, 47, 0, 52),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_helm_chart_dependencies() {
        let text = package_file_fixture("skips-helm-chart-dependencies.txt");
        let dependencies = vec![
            dependency_with(
                Helm,
                "dependencies",
                "zeta",
                unsupported_range(1, 10, 1, 14),
            ),
            dependency_with(
                Helm,
                "dependencies",
                "alpha",
                unsupported_range(3, 10, 3, 15),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert!(!can_sort_dependencies(&dependencies));
        assert_eq!(edits, Vec::new());
    }

    #[test]
    fn skips_sort_when_each_sortable_group_has_one_dependency() {
        let text =
            package_file_fixture("skips-sort-when-each-sortable-group-has-one-dependency.txt");
        let dependencies = vec![
            dependency_with(Npm, "dependencies", "zeta", unsupported_range(2, 4, 2, 15)),
            dependency_with(
                Npm,
                "devDependencies",
                "alpha",
                unsupported_range(5, 4, 5, 16),
            ),
        ];

        let edits = sort_dependency_edits(text, &dependencies);

        assert_eq!(edits, Vec::new());
    }

    fn dependency(name: &str, range: Range) -> Dependency {
        dependency_with(Python, "requirements", name, range)
    }

    fn dependency_with(ecosystem: Ecosystem, group: &str, name: &str, range: Range) -> Dependency {
        Dependency {
            name: name.to_owned(),
            requirement: "1".to_owned(),
            ecosystem,
            group: group.to_owned(),
            hosted_url: None,
            hosted_name: None,
            range,
            requirement_range: range,
            requirement_prefix: String::new(),
            requirement_suffix: String::new(),
            canonical_reference: None,
        }
    }

    fn unsupported_range(
        start_line: u32,
        start_character: u32,
        end_line: u32,
        end_character: u32,
    ) -> Range {
        Range {
            start: Position {
                line: start_line,
                character: start_character,
            },
            end: Position {
                line: end_line,
                character: end_character,
            },
        }
    }
    fn apply_same_line_edits(text: &str, edits: &[TextEdit]) -> String {
        let mut output = text.to_owned();
        let mut ordered = edits.iter().collect::<Vec<_>>();
        ordered.sort_by_key(|edit| Reverse(edit.range.start.character));
        for edit in ordered {
            assert_eq!(edit.range.start.line, 0);
            assert_eq!(edit.range.end.line, 0);
            let start = usize::try_from(edit.range.start.character).unwrap();
            let end = usize::try_from(edit.range.end.character).unwrap();
            output.replace_range(start..end, &edit.new_text);
        }
        output
    }

    fn package_file_fixture(name: &str) -> &'static str {
        crate::support::tests::fixture("tests/fixtures/versionlens-update/src/sort/tests", name)
    }
}
